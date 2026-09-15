// Conformance harness runner — Track B CLI binary.
//
// Usage:
//   rippled-conformance run <scenario.json> [--trace=<path>|--trace=-]
//
// Exit codes:
//   0  success
//   1  simulation error
//   2  usage / IO error

#include <csf/ScenarioLoader.h>
#include <csf/TraceCollector.h>

#include <xrpl/json/json_reader.h>
#include <xrpl/json/json_value.h>

#include <chrono>
#include <cstring>
#include <fstream>
#include <iostream>
#include <memory>
#include <sstream>
#include <string>
#include <string_view>

// ── helpers ───────────────────────────────────────────────────────────────────

namespace {

/** Ostream that discards all output (used when --trace is not supplied). */
struct NullOstream : std::ostream
{
    struct NullBuf : std::streambuf
    {
        int
        overflow(int c) override
        {
            return c;
        }
    } buf;

    NullOstream() : std::ostream(&buf)
    {
    }
};

void
usage()
{
    std::cerr
        << "Usage: rippled-conformance run <scenario.json> "
           "[--trace=<path>|--trace=-]\n"
        << "\n"
        << "  --trace=-        write JSONL trace to stdout\n"
        << "  --trace=<path>   write JSONL trace to file\n";
}

}  // namespace

// ── main ──────────────────────────────────────────────────────────────────────

int
main(int argc, char** argv)
{
    if (argc < 3)
    {
        usage();
        return 2;
    }

    std::string_view const subcommand = argv[1];
    if (subcommand != "run")
    {
        std::cerr << "Unknown subcommand: " << subcommand << "\n";
        usage();
        return 2;
    }

    std::string const scenarioPath = argv[2];

    // Parse optional --trace=<path> argument.
    std::string tracePath;
    for (int i = 3; i < argc; ++i)
    {
        std::string_view const arg = argv[i];
        if (arg.substr(0, 8) == "--trace=")
            tracePath = std::string(arg.substr(8));
    }

    // ── Read and parse scenario JSON ──────────────────────────────────────────
    std::ifstream scenarioFile(scenarioPath);
    if (!scenarioFile.is_open())
    {
        std::cerr << "Error: cannot open scenario file: " << scenarioPath << "\n";
        return 2;
    }

    std::ostringstream buf;
    buf << scenarioFile.rdbuf();
    std::string const scenarioJson = buf.str();

    json::Value root;
    json::Reader reader;
    if (!reader.parse(scenarioJson, root))
    {
        std::cerr << "Error: failed to parse scenario JSON: "
                  << reader.getFormattedErrorMessages() << "\n";
        return 2;
    }

    // ── Set up trace output stream ────────────────────────────────────────────
    NullOstream nullStream;
    std::unique_ptr<std::ofstream> traceFilePtr;
    std::ostream* traceStream = nullptr;

    if (tracePath.empty())
    {
        traceStream = &nullStream;
    }
    else if (tracePath == "-")
    {
        traceStream = &std::cout;
    }
    else
    {
        traceFilePtr = std::make_unique<std::ofstream>(tracePath);
        if (!traceFilePtr->is_open())
        {
            std::cerr << "Error: cannot open trace output file: " << tracePath << "\n";
            return 2;
        }
        traceStream = traceFilePtr.get();
    }

    // ── Load scenario and run simulation ──────────────────────────────────────
    xrpl::test::csf::TraceCollector trace(*traceStream);

    auto sim = xrpl::test::csf::loadScenario(root, trace);

    std::uint32_t const durationMs = root.isMember("duration_ms")
        ? root["duration_ms"].asUInt()
        : 60000u;

    // Not `sim->run(duration)`: `Sim::run` calls `Peer::start()` on every peer,
    // but `loadScenario` has already started them (`start(0ms)`, so the first
    // timer fires at t=0 like the Rust sim). A second `start()` would add a
    // second `timerEntry` chain per peer (two ticks per second, and an extra
    // `shouldCloseLedger` check right after each `startRound`) plus a duplicate
    // `startRound` at t=0. `targetLedgers` already defaults to "unbounded".
    sim->scheduler.stepFor(std::chrono::milliseconds(durationMs));

    if (!tracePath.empty() && tracePath != "-")
        std::cerr << "Trace written to " << tracePath << "\n";

    return 0;
}
