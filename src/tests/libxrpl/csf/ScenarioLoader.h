#pragma once

#include <csf/Sim.h>
#include <csf/TraceCollector.h>

#include <xrpl/json/json_value.h>

#include <algorithm>
#include <chrono>
#include <cmath>
#include <functional>
#include <memory>
#include <string>
#include <unordered_map>
#include <vector>

namespace xrpl::test::csf {

/** Load a scenario from a JSON value and configure a Sim for it.

    The returned Sim has all peers created, topology wired, events scheduled,
    and peers started (via Peer::start()).  The caller only needs to call
    sim->run(duration) to execute the scenario.

    @param scenario  Parsed scenario JSON object (§3 format).
    @param trace     Trace collector that receives conformance events.
    @return          Owning pointer to the configured (but not yet run) Sim.
*/
inline std::unique_ptr<Sim>
loadScenario(json::Value const& scenario, TraceCollector& trace)
{
    auto sim = std::make_unique<Sim>();

    // ── Create peers ──────────────────────────────────────────────────────────
    // Scenario peers must form a contiguous 0-based ID range; createGroup
    // assigns IDs in that order.
    json::Value const& peersArr = scenario["peers"];
    std::uint32_t const numPeers = peersArr.size();

    auto peerGroup = sim->createGroup(numPeers);

    // Build id→Peer* lookup.
    std::unordered_map<std::uint32_t, Peer*> peerById;
    for (std::uint32_t i = 0; i < numPeers; ++i)
        peerById[i] = peerGroup[i];

    // ConsensusParms fields are all const — overrides not supported at runtime.
    // Scenario `parms` is intentionally ignored; all peers use default parms.
    // (Track C parms parity is a future work item.)

    // ── Per-peer configuration ────────────────────────────────────────────────
    for (std::uint32_t idx = 0; idx < numPeers; ++idx)
    {
        json::Value const& pc = peersArr[idx];
        std::uint32_t const peerId = pc["id"].asUInt();
        Peer* peer = peerById[peerId];

        if (pc.isMember("proposing"))
            peer->runAsValidator = pc["proposing"].asBool();

        if (pc.isMember("processing_delay_ms"))
            peer->delays.ledgerAccept =
                std::chrono::milliseconds(pc["processing_delay_ms"].asUInt());
    }

    // ── Trust (UNL) setup ─────────────────────────────────────────────────────
    // Done after all peers are in peerById so cross-references work.
    for (std::uint32_t idx = 0; idx < numPeers; ++idx)
    {
        json::Value const& pc = peersArr[idx];
        if (!pc.isMember("unl"))
            continue;

        std::uint32_t const peerId = pc["id"].asUInt();
        Peer* peer = peerById[peerId];
        json::Value const& unl = pc["unl"];

        for (std::uint32_t u = 0; u < unl.size(); ++u)
        {
            std::uint32_t const trusted = unl[u].asUInt();
            if (auto it = peerById.find(trusted); it != peerById.end())
                peer->trust(*it->second);
        }
    }

    // ── Network edges ─────────────────────────────────────────────────────────
    if (scenario.isMember("network") && !scenario["network"].isNull() &&
        scenario["network"].size() > 0)
    {
        json::Value const& netArr = scenario["network"];
        for (std::uint32_t i = 0; i < netArr.size(); ++i)
        {
            json::Value const& edge = netArr[i];
            std::uint32_t const from = edge["from"].asUInt();
            std::uint32_t const to = edge["to"].asUInt();
            std::uint32_t const delayMs =
                edge.isMember("delay_ms") ? edge["delay_ms"].asUInt() : 50u;
            auto const delay = std::chrono::milliseconds(delayMs);

            auto fit = peerById.find(from);
            auto tit = peerById.find(to);
            if (fit != peerById.end() && tit != peerById.end())
            {
                // Bidirectional
                fit->second->connect(*tit->second, delay);
                tit->second->connect(*fit->second, delay);
            }
        }
    }
    else
    {
        // Default: fully connected with 50 ms delay.
        using namespace std::chrono_literals;
        peerGroup.connect(peerGroup, 50ms);
    }

    // ── Scripted events ───────────────────────────────────────────────────────
    if (scenario.isMember("events"))
    {
        json::Value const& evts = scenario["events"];
        for (std::uint32_t i = 0; i < evts.size(); ++i)
        {
            json::Value const& ev = evts[i];
            std::uint32_t const tMs = ev["t"].asUInt();
            std::string const kind = ev["kind"].asString();
            auto const absTime = SimTime{} + std::chrono::milliseconds(tMs);

            if (kind == "submit_tx")
            {
                std::uint32_t const nodeId = ev["node"].asUInt();
                std::string const txStr = ev["tx"].asString();
                // Deterministically map the tx label string to a uint32_t ID.
                std::uint32_t const txHash =
                    static_cast<std::uint32_t>(std::hash<std::string>{}(txStr));
                sim->scheduler.at(absTime, [peer = peerById[nodeId], txHash]() {
                    peer->submit(Tx{txHash});
                });
            }
            else if (kind == "disconnect")
            {
                std::uint32_t const from = ev["from"].asUInt();
                std::uint32_t const to = ev["to"].asUInt();
                sim->scheduler.at(
                    absTime,
                    [pFrom = peerById[from], pTo = peerById[to]]() {
                        pFrom->disconnect(*pTo);
                        pTo->disconnect(*pFrom);
                    });
            }
            else if (kind == "reconnect")
            {
                std::uint32_t const from = ev["from"].asUInt();
                std::uint32_t const to = ev["to"].asUInt();
                sim->scheduler.at(
                    absTime,
                    [pFrom = peerById[from], pTo = peerById[to]]() {
                        pFrom->connect(*pTo, std::chrono::milliseconds(50));
                        pTo->connect(*pFrom, std::chrono::milliseconds(50));
                    });
            }
            else if (kind == "set_trust")
            {
                std::uint32_t const nodeId = ev["node"].asUInt();
                std::vector<std::uint32_t> trustIds;
                json::Value const& trustArr = ev["trust"];
                for (std::uint32_t u = 0; u < trustArr.size(); ++u)
                    trustIds.push_back(trustArr[u].asUInt());

                // Capture peerById by value so the lambda owns the map copy.
                sim->scheduler.at(
                    absTime,
                    [peer = peerById[nodeId], trustIds, peerById]() {
                        // Remove existing trust from all peers.
                        for (auto const& [id, p] : peerById)
                            peer->untrust(*p);
                        // Grant new trust.
                        for (auto const id : trustIds)
                        {
                            if (auto it = peerById.find(id); it != peerById.end())
                                peer->trust(*it->second);
                        }
                    });
            }
            else if (kind == "bow_out")
            {
                std::uint32_t const nodeId = ev["node"].asUInt();
                sim->scheduler.at(absTime, [peer = peerById[nodeId]]() {
                    // Emit the bow-out event so TraceCollector records it,
                    // then broadcast a leave proposal to peers.
                    peer->issue(BowOut{.seq = Proposal::kSeqLeave});

                    Proposal leaveProposal(
                        peer->lastClosedLedger.id(),
                        Proposal::kSeqLeave,
                        TxSet::calcID(peer->openTxs),
                        peer->now(),
                        peer->now(),
                        peer->id);
                    peer->share(leaveProposal);
                });
            }
            // Other event kinds (advance_clock, inject_validation, etc.)
            // are not implemented in this version.
        }
    }

    // ── Start peers ───────────────────────────────────────────────────────────
    // Fire the first timer at t=0 (matching Rust's next_timer_ms=0 init).
    for (std::uint32_t i = 0; i < numPeers; ++i)
        peerGroup[i]->start(std::chrono::milliseconds{0});

    // Register the trace collector AFTER all start() calls so that the initial
    // Observing→Proposing mode change (emitted during startRound() inside start())
    // is not captured — matching Rust's behavior where the trace is wired after
    // load_scenario() returns.
    sim->collectors.add(trace);
    for (std::uint32_t i = 0; i < numPeers; ++i)
        peerGroup[i]->traceCollector_ = &trace;

    return sim;
}

}  // namespace xrpl::test::csf
