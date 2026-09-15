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

namespace detail {

/** Convert a scenario millisecond timestamp to the csf NetClock.

    Mirrors the unskewed part of Peer::now(): sim time + 86400 s so that
    NetClock subtractions inside consensus stay positive.  The Rust sim's
    `net_time` applies the same offset, so injected timestamps line up.
*/
inline NetClock::time_point
scenarioMsToNetClock(std::uint64_t ms)
{
    using namespace std::chrono;
    using namespace std::chrono_literals;
    return NetClock::time_point(
        duration_cast<NetClock::duration>(milliseconds(ms) + 86400s));
}

/** Map a scenario ledger-id string onto a csf Ledger::ID.

    "GENESIS" and an all-zero hex string both denote the genesis ledger
    (Ledger::ID{0}); a decimal string is taken literally (csf ledger IDs
    are TaggedInteger<uint32_t>); anything else is hashed so that distinct
    labels stay distinct without colliding with real ledger IDs by accident.
*/
inline Ledger::ID
scenarioLedgerId(std::string const& s)
{
    if (s == "GENESIS" || s.empty() ||
        std::ranges::all_of(s, [](char c) { return c == '0'; }))
        return Ledger::ID{0};
    if (std::ranges::all_of(s, [](unsigned char c) { return std::isdigit(c); }))
        return Ledger::ID{static_cast<std::uint32_t>(std::stoul(s))};
    return Ledger::ID{static_cast<std::uint32_t>(std::hash<std::string>{}(s))};
}

/** Map an opaque scenario tx-set label onto a csf TxSet::ID.

    Same deterministic std::hash scheme `submit_tx` uses for tx labels.  A
    label never equals the ID of a real TxSet, so a synthetic position (e.g.
    "byzantine_empty") disputes every honest position — matching the Rust
    sim, whose SimTxSetId carries the raw label.
*/
inline TxSet::ID
scenarioTxSetId(std::string const& label)
{
    return static_cast<TxSet::ID>(std::hash<std::string>{}(label));
}

}  // namespace detail

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
            else if (kind == "inject_proposal")
            {
                // Deliver a synthetic proposal from `proposer` straight into
                // `node`, bypassing the network layer.  Runs the same
                // Peer::handle(Proposal) path a network delivery would, so
                // the receiver applies its trust check, per-ledger position
                // dedup, emits peer_proposal_received, and lets the engine's
                // per-peer propose-seq monotonicity check drop duplicates.
                std::uint32_t const nodeId = ev["node"].asUInt();
                std::uint32_t const proposer = ev["proposer"].asUInt();
                std::uint32_t const seq = ev["seq"].asUInt();
                TxSet::ID const txSetId =
                    detail::scenarioTxSetId(ev["tx_set"].asString());
                Ledger::ID const prevLedger =
                    detail::scenarioLedgerId(ev["prev_ledger"].asString());
                NetClock::time_point const closeTime =
                    detail::scenarioMsToNetClock(ev["close_time"].asUInt());

                auto const nit = peerById.find(nodeId);
                auto const pit = peerById.find(proposer);
                if (nit == peerById.end() || pit == peerById.end())
                    continue;

                sim->scheduler.at(
                    absTime,
                    [peer = nit->second,
                     proposerId = pit->second->id,
                     seq,
                     txSetId,
                     prevLedger,
                     closeTime,
                     tMs]() {
                        // seenTime uses the unskewed sim clock, like the
                        // Rust sim's `net_time(sim_ms)`.
                        Proposal const p(
                            prevLedger,
                            seq,
                            txSetId,
                            closeTime,
                            detail::scenarioMsToNetClock(tMs),
                            proposerId);
                        peer->handle(p);
                    });
            }
            else if (kind == "inject_validation")
            {
                // Deliver a synthetic validation from `validator` straight
                // into `node`.  Runs Peer::handle(Validation): trust check,
                // then Validations::add, whose isCurrent() window decides
                // acceptance.  Emits a `validation_received` trace event
                // carrying the outcome so it can be cross-diffed against the
                // Rust sim's InjectValidation handling.
                std::uint32_t const nodeId = ev["node"].asUInt();
                std::uint32_t const validator = ev["validator"].asUInt();
                std::string const ledgerStr = ev["ledger"].asString();
                std::uint32_t const seq = ev["seq"].asUInt();
                std::uint64_t const signTimeMs =
                    ev.isMember("sign_time") ? ev["sign_time"].asUInt() : 0u;

                auto const nit = peerById.find(nodeId);
                auto const vit = peerById.find(validator);
                if (nit == peerById.end() || vit == peerById.end())
                    continue;

                sim->scheduler.at(
                    absTime,
                    [peer = nit->second,
                     validatorId = vit->second->id,
                     ledgerStr,
                     ledgerId = detail::scenarioLedgerId(ledgerStr),
                     seq,
                     signTimeMs,
                     &trace]() {
                        NetClock::time_point const signTime =
                            detail::scenarioMsToNetClock(signTimeMs);
                        Validation const v{
                            ledgerId,
                            Ledger::Seq{seq},
                            signTime,
                            NetClock::time_point{},  // seen: set on add
                            PeerKey{validatorId, 0},
                            validatorId,
                            /*full=*/true};

                        // Classify a rejection the same way Validations::isCurrent
                        // does, evaluated against the receiver's clock.
                        ValidationParms const vp{};
                        NetClock::time_point const now = peer->now();
                        std::string reason;
                        if (!(signTime < now + vp.validationCurrentWall))
                            reason = "too_early";
                        else if (!(signTime > now - vp.validationCurrentEarly))
                            reason = "too_late";

                        bool const accepted = peer->handle(v);
                        trace.onValidationReceived(
                            peer->id,
                            peer->scheduler.now(),
                            validatorId,
                            ledgerStr,
                            seq,
                            signTimeMs,
                            accepted,
                            accepted ? std::string{} : reason);
                    });
            }
            // Other event kinds (advance_clock, etc.) are not implemented in
            // this version.
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
