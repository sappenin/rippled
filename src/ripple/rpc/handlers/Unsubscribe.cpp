//------------------------------------------------------------------------------
/*
    This file is part of rippled: https://github.com/ripple/rippled
    Copyright (c) 2012-2014 Ripple Labs Inc.

    Permission to use, copy, modify, and/or distribute this software for any
    purpose  with  or without fee is hereby granted, provided that the above
    copyright notice and this permission notice appear in all copies.

    THE  SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
    WITH  REGARD  TO  THIS  SOFTWARE  INCLUDING  ALL  IMPLIED  WARRANTIES  OF
    MERCHANTABILITY  AND  FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
    ANY  SPECIAL ,  DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
    WHATSOEVER  RESULTING  FROM  LOSS  OF USE, DATA OR PROFITS, WHETHER IN AN
    ACTION  OF  CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF
    OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
*/
//==============================================================================

#include <ripple/app/misc/NetworkOPs.h>
#include <ripple/basics/Log.h>
#include <ripple/net/RPCErr.h>
#include <ripple/protocol/ErrorCodes.h>
#include <ripple/protocol/jss.h>
#include <ripple/rpc/Context.h>
#include <ripple/rpc/Role.h>
#include <ripple/rpc/impl/RPCHelpers.h>

namespace ripple {

Json::Value
doUnsubscribe(RPC::JsonContext& context)
{
    InfoSub::pointer ispSub;
    Json::Value jvResult(Json::objectValue);
    bool removeUrl{false};

    if (!context.infoSub && !context.params.isMember(jss::url))
    {
        // Must be a JSON-RPC call.
        return rpcError(rpcINVALID_PARAMS);
    }

    if (context.params.isMember(jss::url))
    {
        if (context.role != Role::ADMIN)
            return rpcError(rpcNO_PERMISSION);

        std::string strUrl = context.params[jss::url].asString();
        ispSub = context.netOps.findRpcSub(strUrl);
        if (!ispSub)
            return jvResult;
        removeUrl = true;
    }
    else
    {
        ispSub = context.infoSub;
    }

    if (context.params.isMember(jss::streams))
    {
        if (!context.params[jss::streams].isArray())
            return rpcError(rpcINVALID_PARAMS);

        for (auto& it : context.params[jss::streams])
        {
            if (!it.isString())
                return rpcError(rpcSTREAM_MALFORMED);

            std::string streamName = it.asString();
            if (streamName == "server")
            {
                context.netOps.unsubServer(ispSub->getSeq());
            }
            else if (streamName == "ledger")
            {
                context.netOps.unsubLedger(ispSub->getSeq());
            }
            else if (streamName == "manifests")
            {
                context.netOps.unsubManifests(ispSub->getSeq());
            }
            else if (streamName == "transactions")
            {
                context.netOps.unsubTransactions(ispSub->getSeq());
            }
            else if (
                streamName == "transactions_proposed" ||
                streamName == "rt_transactions")  // DEPRECATED
            {
                context.netOps.unsubRTTransactions(ispSub->getSeq());
            }
            else if (streamName == "validations")
            {
                context.netOps.unsubValidations(ispSub->getSeq());
            }
            else if (streamName == "peer_status")
            {
                context.netOps.unsubPeerStatus(ispSub->getSeq());
            }
            else if (streamName == "consensus")
            {
                context.netOps.unsubConsensus(ispSub->getSeq());
            }
            else
            {
                return rpcError(rpcSTREAM_MALFORMED);
            }
        }
    }

    auto accountsProposed = context.params.isMember(jss::accounts_proposed)
        ? jss::accounts_proposed
        : jss::rt_accounts;  // DEPRECATED
    if (context.params.isMember(accountsProposed))
    {
        if (!context.params[accountsProposed].isArray())
            return rpcError(rpcINVALID_PARAMS);

        auto ids = RPC::parseAccountIds(context.params[accountsProposed]);
        if (ids.empty())
            return rpcError(rpcACT_MALFORMED);
        context.netOps.unsubAccount(ispSub, ids, true);
    }

    if (context.params.isMember(jss::accounts))
    {
        if (!context.params[jss::accounts].isArray())
            return rpcError(rpcINVALID_PARAMS);

        auto ids = RPC::parseAccountIds(context.params[jss::accounts]);
        if (ids.empty())
            return rpcError(rpcACT_MALFORMED);
        context.netOps.unsubAccount(ispSub, ids, false);
    }

    if (context.params.isMember(jss::account_history_tx_stream))
    {
        auto const& req = context.params[jss::account_history_tx_stream];
        if (!req.isMember(jss::account) || !req[jss::account].isString())
            return rpcError(rpcINVALID_PARAMS);

        auto const id = parseBase58<AccountID>(req[jss::account].asString());
        if (!id)
            return rpcError(rpcINVALID_PARAMS);

        bool stopHistoryOnly = false;
        if (req.isMember(jss::stop_history_tx_only))
        {
            if (!req[jss::stop_history_tx_only].isBool())
                return rpcError(rpcINVALID_PARAMS);
            stopHistoryOnly = req[jss::stop_history_tx_only].asBool();
        }
        context.netOps.unsubAccountHistory(ispSub, *id, stopHistoryOnly);

        JLOG(context.j.debug())
            << "doUnsubscribe: account_history_tx_stream: " << toBase58(*id)
            << " stopHistoryOnly=" << (stopHistoryOnly ? "true" : "false");
    }

    if (removeUrl)
    {
        context.netOps.tryRemoveRpcSub(context.params[jss::url].asString());
    }

    return jvResult;
}

}  // namespace ripple
