#pragma once

#include <csf/SimTime.h>
#include <csf/events.h>

#include <xrpl/basics/strHex.h>
#include <xrpl/consensus/ConsensusParms.h>
#include <xrpl/consensus/ConsensusTypes.h>
#include <xrpl/json/json_value.h>
#include <xrpl/json/json_writer.h>

#include <algorithm>
#include <cctype>
#include <chrono>
#include <optional>
#include <ostream>
#include <string>
#include <vector>

namespace xrpl::test::csf {

// ── Data-transfer structs for the tick snapshot ──────────────────────────────

struct DisputeEntry {
    std::string tx_id;
    bool ours;
    int yays;
    int nays;
    std::string av_state;
    std::size_t av_counter;
    std::size_t vote_counter;
};

struct PositionEntry {
    std::string prev;
    std::string tx_set;
    std::int64_t close_time_secs;
    std::uint32_t seq;
    std::string sign_hash;  // lowercase hex
};

struct CsTickSnapshot {
    ConsensusPhase phase;
    ConsensusMode mode;
    std::string lcl_id;
    std::uint32_t lcl_seq;
    std::string fully_validated_id;
    std::int64_t prev_round_ms;
    int converge_pct;
    std::optional<PositionEntry> position;
    // sorted ascending by time
    std::vector<std::pair<std::int64_t, int>> close_times;
    // sorted by tx_id string
    std::vector<DisputeEntry> disputes;
    std::size_t validations_current;
    std::size_t validations_trusted;
};

// ── TraceCollector ────────────────────────────────────────────────────────────

class TraceCollector
{
public:
    explicit TraceCollector(std::ostream& sink) : sink_{sink}
    {
    }

    // ── tick snapshot ────────────────────────────────────────────────────────
    void
    onTick(PeerID who, SimTime when, CsTickSnapshot const& snap)
    {
        json::Value v(json::ValueType::Object);
        v["k"] = "tick";
        v["t"] = static_cast<json::UInt>(toMs(when));
        v["node"] = static_cast<json::UInt>(peerToUInt(who));
        v["phase"] = phaseStr(snap.phase);
        v["mode"] = modeStr(snap.mode);
        v["lcl"] = snap.lcl_id;
        v["lcl_seq"] = static_cast<json::UInt>(snap.lcl_seq);
        v["fully_validated"] = snap.fully_validated_id;
        v["prev_round_ms"] = static_cast<json::UInt>(
            static_cast<std::uint32_t>(snap.prev_round_ms));
        v["converge_pct"] = snap.converge_pct;

        if (snap.position)
        {
            json::Value pos(json::ValueType::Object);
            pos["prev"] = snap.position->prev;
            pos["tx_set"] = snap.position->tx_set;
            pos["close_time"] = static_cast<json::UInt>(
                static_cast<std::uint32_t>(snap.position->close_time_secs));
            pos["seq"] = static_cast<json::UInt>(snap.position->seq);
            pos["sign_hash"] = snap.position->sign_hash;
            v["position"] = pos;
        }

        json::Value ct(json::ValueType::Array);
        for (auto const& [secs, count] : snap.close_times)
        {
            json::Value pair(json::ValueType::Array);
            pair.append(static_cast<json::UInt>(static_cast<std::uint32_t>(secs)));
            pair.append(count);
            ct.append(pair);
        }
        v["close_times"] = ct;

        json::Value disputes(json::ValueType::Array);
        for (auto const& d : snap.disputes)
        {
            json::Value de(json::ValueType::Object);
            de["tx"] = d.tx_id;
            de["ours"] = d.ours;
            de["yays"] = d.yays;
            de["nays"] = d.nays;
            de["av_state"] = d.av_state;
            de["av_counter"] = static_cast<json::UInt>(
                static_cast<std::uint32_t>(d.av_counter));
            de["vote_counter"] = static_cast<json::UInt>(
                static_cast<std::uint32_t>(d.vote_counter));
            disputes.append(de);
        }
        v["disputes"] = disputes;

        v["trie"] = json::Value();

        json::Value vals(json::ValueType::Object);
        vals["current"] = static_cast<json::UInt>(
            static_cast<std::uint32_t>(snap.validations_current));
        vals["trusted"] = static_cast<json::UInt>(
            static_cast<std::uint32_t>(snap.validations_trusted));
        v["validations"] = vals;

        emit(v);
    }

    // ── §4.2 event overloads ─────────────────────────────────────────────────

    void
    on(PeerID who, SimTime when, Propose const& e)
    {
        using std::to_string;
        json::Value v = baseEvent(who, when, "propose");
        v["seq"] = static_cast<json::UInt>(e.proposal.proposeSeq());
        v["position"] = to_string(e.proposal.position());
        v["prev"] = to_string(e.proposal.prevLedger());
        v["close_time"] = static_cast<json::UInt>(static_cast<std::uint32_t>(
            e.proposal.closeTime().time_since_epoch().count()));
        // CSF TaggedInteger types are not sha512Half-hashable; sign_hash is
        // intentionally omitted (use --ignore event.sign_hash in cross-diff).
        v["sign_hash"] = "";
        emit(v);
    }

    void
    on(PeerID who, SimTime when, PeerProposalReceived const& e)
    {
        using std::to_string;
        json::Value v = baseEvent(who, when, "peer_proposal_received");
        v["from"] = static_cast<json::UInt>(peerToUInt(e.from));
        v["seq"] = static_cast<json::UInt>(e.proposal.proposeSeq());
        v["prev"] = to_string(e.proposal.prevLedger());
        v["position"] = to_string(e.proposal.position());
        v["close_time"] = static_cast<json::UInt>(static_cast<std::uint32_t>(
            e.proposal.closeTime().time_since_epoch().count()));
        emit(v);
    }

    void
    on(PeerID who, SimTime when, TxSetReceived const& e)
    {
        json::Value v = baseEvent(who, when, "tx_set_received");
        v["tx_set"] = std::to_string(e.id);
        v["size"] = static_cast<json::UInt>(
            static_cast<std::uint32_t>(e.size));
        emit(v);
    }

    void
    on(PeerID who, SimTime when, ModeChange const& e)
    {
        json::Value v = baseEvent(who, when, "mode_change");
        v["from"] = modeStr(e.from);
        v["to"] = modeStr(e.to);
        emit(v);
    }

    void
    on(PeerID who, SimTime when, BowOut const& e)
    {
        json::Value v = baseEvent(who, when, "bow_out");
        v["seq"] = static_cast<json::UInt>(e.seq);
        emit(v);
    }

    void
    on(PeerID who, SimTime when, CloseLedger const& e)
    {
        json::Value v = baseEvent(who, when, "close_ledger");
        // open_time_ms: not directly available from the event, emit 0
        v["open_time_ms"] = static_cast<json::UInt>(0);
        v["any_tx"] = !e.txs.empty();
        emit(v);
    }

    void
    on(PeerID who, SimTime when, AcceptLedger const& e)
    {
        using std::to_string;
        json::Value v = baseEvent(who, when, "accept_ledger");
        v["ledger"] = to_string(e.ledger.id());
        // ConsensusState not available here; use "Yes" as default
        v["state"] = "Yes";
        emit(v);
    }

    void
    on(PeerID who, SimTime when, FullyValidateLedger const& e)
    {
        using std::to_string;
        json::Value v = baseEvent(who, when, "fully_validate");
        v["ledger"] = to_string(e.ledger.id());
        emit(v);
    }

    // `validation_received`: emitted by ScenarioLoader's `inject_validation`
    // handler (csf has no event struct for validation receipt; network-delivered
    // validations emit nothing, matching the Rust sim).  `reason` is only
    // written when non-empty, mirroring the Rust emitter.
    void
    onValidationReceived(
        PeerID who,
        SimTime when,
        PeerID validator,
        std::string const& ledger,
        std::uint32_t seq,
        std::uint64_t signTimeMs,
        bool accepted,
        std::string const& reason)
    {
        json::Value v = baseEvent(who, when, "validation_received");
        v["validator"] = static_cast<json::UInt>(peerToUInt(validator));
        v["ledger"] = ledger;
        v["seq"] = static_cast<json::UInt>(seq);
        v["sign_time"] = static_cast<json::UInt>(
            static_cast<std::uint32_t>(signTimeMs));
        v["accepted"] = accepted;
        if (!reason.empty())
            v["reason"] = reason;
        emit(v);
    }

    // No-op for all other events (Share, Relay, Receive, SubmitTx, StartRound,
    // WrongPrevLedger) — not part of the §4.2 trace surface.
    template <class E>
    void
    on(PeerID, SimTime, E const&)
    {
    }

private:
    std::ostream& sink_;

    std::int64_t
    toMs(SimTime t) const
    {
        return std::chrono::duration_cast<std::chrono::milliseconds>(
                   t.time_since_epoch())
            .count();
    }

    static std::uint32_t
    peerToUInt(PeerID p)
    {
        using std::to_string;
        return static_cast<std::uint32_t>(std::stoul(to_string(p)));
    }

    void
    emit(json::Value v)
    {
        sink_ << json::Compact{std::move(v)} << '\n';
    }

    json::Value
    baseEvent(PeerID who, SimTime when, std::string const& kind)
    {
        json::Value v(json::ValueType::Object);
        v["k"] = "event";
        v["t"] = static_cast<json::UInt>(static_cast<std::uint32_t>(toMs(when)));
        v["node"] = static_cast<json::UInt>(peerToUInt(who));
        v["e"] = kind;
        return v;
    }

    static std::string
    hashToLowerHex(uint256 const& h)
    {
        auto hex = strHex(h);
        std::transform(
            hex.begin(), hex.end(), hex.begin(), [](unsigned char c) {
                return std::tolower(c);
            });
        return hex;
    }

    static std::string
    phaseStr(ConsensusPhase p)
    {
        switch (p)
        {
            case ConsensusPhase::Open:
                return "Open";
            case ConsensusPhase::Establish:
                return "Establish";
            case ConsensusPhase::Accepted:
                return "Accepted";
        }
        return "Unknown";
    }

    static std::string
    modeStr(ConsensusMode m)
    {
        switch (m)
        {
            case ConsensusMode::Proposing:
                return "Proposing";
            case ConsensusMode::Observing:
                return "Observing";
            case ConsensusMode::WrongLedger:
                return "WrongLedger";
            case ConsensusMode::SwitchedLedger:
                return "SwitchedLedger";
        }
        return "Unknown";
    }

    static std::string
    avStateStr(ConsensusParms::AvalancheState s)
    {
        switch (s)
        {
            case ConsensusParms::AvalancheState::Init:
                return "Init";
            case ConsensusParms::AvalancheState::Mid:
                return "Mid";
            case ConsensusParms::AvalancheState::Late:
                return "Late";
            case ConsensusParms::AvalancheState::Stuck:
                return "Stuck";
        }
        return "Unknown";
    }
};

}  // namespace xrpl::test::csf
