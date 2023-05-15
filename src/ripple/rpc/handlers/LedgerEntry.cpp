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

#include <ripple/app/main/Application.h>
#include <ripple/basics/StringUtilities.h>
#include <ripple/basics/strHex.h>
#include <ripple/ledger/ReadView.h>
#include <ripple/net/RPCErr.h>
#include <ripple/protocol/ErrorCodes.h>
#include <ripple/protocol/Indexes.h>
#include <ripple/protocol/jss.h>
#include <ripple/rpc/Context.h>
#include <ripple/rpc/GRPCHandlers.h>
#include <ripple/rpc/impl/RPCHelpers.h>

namespace ripple {

// {
//   ledger_hash : <ledger>
//   ledger_index : <ledger_index>
//   ...
// }
Json::Value
doLedgerEntry(RPC::JsonContext& context)
{
    std::shared_ptr<ReadView const> lpLedger;
    auto jvResult = RPC::lookupLedger(lpLedger, context);

    if (!lpLedger)
        return jvResult;

    uint256 uNodeIndex;
    bool bNodeBinary = false;
    LedgerEntryType expectedType = ltANY;

    if (context.params.isMember(jss::index))
    {
        if (!uNodeIndex.parseHex(context.params[jss::index].asString()))
        {
            uNodeIndex = beast::zero;
            jvResult[jss::error] = "malformedRequest";
        }
    }
    else if (context.params.isMember(jss::account_root))
    {
        expectedType = ltACCOUNT_ROOT;
        auto const account = parseBase58<AccountID>(
            context.params[jss::account_root].asString());
        if (!account || account->isZero())
            jvResult[jss::error] = "malformedAddress";
        else
            uNodeIndex = keylet::account(*account).key;
    }
    else if (context.params.isMember(jss::deposit_preauth))
    {
        expectedType = ltDEPOSIT_PREAUTH;

        if (!context.params[jss::deposit_preauth].isObject())
        {
            if (!context.params[jss::deposit_preauth].isString() ||
                !uNodeIndex.parseHex(
                    context.params[jss::deposit_preauth].asString()))
            {
                uNodeIndex = beast::zero;
                jvResult[jss::error] = "malformedRequest";
            }
        }
        else if (
            !context.params[jss::deposit_preauth].isMember(jss::owner) ||
            !context.params[jss::deposit_preauth][jss::owner].isString() ||
            !context.params[jss::deposit_preauth].isMember(jss::authorized) ||
            !context.params[jss::deposit_preauth][jss::authorized].isString())
        {
            jvResult[jss::error] = "malformedRequest";
        }
        else
        {
            auto const owner = parseBase58<AccountID>(
                context.params[jss::deposit_preauth][jss::owner].asString());

            auto const authorized = parseBase58<AccountID>(
                context.params[jss::deposit_preauth][jss::authorized]
                    .asString());

            if (!owner)
                jvResult[jss::error] = "malformedOwner";
            else if (!authorized)
                jvResult[jss::error] = "malformedAuthorized";
            else
                uNodeIndex = keylet::depositPreauth(*owner, *authorized).key;
        }
    }
    else if (context.params.isMember(jss::directory))
    {
        expectedType = ltDIR_NODE;
        if (context.params[jss::directory].isNull())
        {
            jvResult[jss::error] = "malformedRequest";
        }
        else if (!context.params[jss::directory].isObject())
        {
            if (!uNodeIndex.parseHex(context.params[jss::directory].asString()))
            {
                uNodeIndex = beast::zero;
                jvResult[jss::error] = "malformedRequest";
            }
        }
        else if (
            context.params[jss::directory].isMember(jss::sub_index) &&
            !context.params[jss::directory][jss::sub_index].isIntegral())
        {
            jvResult[jss::error] = "malformedRequest";
        }
        else
        {
            std::uint64_t uSubIndex =
                context.params[jss::directory].isMember(jss::sub_index)
                ? context.params[jss::directory][jss::sub_index].asUInt()
                : 0;

            if (context.params[jss::directory].isMember(jss::dir_root))
            {
                uint256 uDirRoot;

                if (context.params[jss::directory].isMember(jss::owner))
                {
                    // May not specify both dir_root and owner.
                    jvResult[jss::error] = "malformedRequest";
                }
                else if (!uDirRoot.parseHex(
                             context.params[jss::directory][jss::dir_root]
                                 .asString()))
                {
                    uNodeIndex = beast::zero;
                    jvResult[jss::error] = "malformedRequest";
                }
                else
                {
                    uNodeIndex = keylet::page(uDirRoot, uSubIndex).key;
                }
            }
            else if (context.params[jss::directory].isMember(jss::owner))
            {
                auto const ownerID = parseBase58<AccountID>(
                    context.params[jss::directory][jss::owner].asString());

                if (!ownerID)
                {
                    jvResult[jss::error] = "malformedAddress";
                }
                else
                {
                    uNodeIndex =
                        keylet::page(keylet::ownerDir(*ownerID), uSubIndex).key;
                }
            }
            else
            {
                jvResult[jss::error] = "malformedRequest";
            }
        }
    }
    else if (context.params.isMember(jss::offer))
    {
        expectedType = ltOFFER;
        if (!context.params[jss::offer].isObject())
        {
            if (!uNodeIndex.parseHex(context.params[jss::offer].asString()))
            {
                uNodeIndex = beast::zero;
                jvResult[jss::error] = "malformedRequest";
            }
        }
        else if (
            !context.params[jss::offer].isMember(jss::account) ||
            !context.params[jss::offer].isMember(jss::seq) ||
            !context.params[jss::offer][jss::seq].isIntegral())
        {
            jvResult[jss::error] = "malformedRequest";
        }
        else
        {
            auto const id = parseBase58<AccountID>(
                context.params[jss::offer][jss::account].asString());
            if (!id)
                jvResult[jss::error] = "malformedAddress";
            else
                uNodeIndex =
                    keylet::offer(
                        *id, context.params[jss::offer][jss::seq].asUInt())
                        .key;
        }
    }
    else if (context.params.isMember(jss::payment_channel))
    {
        expectedType = ltPAYCHAN;

        if (!uNodeIndex.parseHex(
                context.params[jss::payment_channel].asString()))
        {
            uNodeIndex = beast::zero;
            jvResult[jss::error] = "malformedRequest";
        }
    }
    else if (context.params.isMember(jss::ripple_state))
    {
        expectedType = ltRIPPLE_STATE;
        Currency uCurrency;
        Json::Value jvRippleState = context.params[jss::ripple_state];

        if (!jvRippleState.isObject() ||
            !jvRippleState.isMember(jss::currency) ||
            !jvRippleState.isMember(jss::accounts) ||
            !jvRippleState[jss::accounts].isArray() ||
            2 != jvRippleState[jss::accounts].size() ||
            !jvRippleState[jss::accounts][0u].isString() ||
            !jvRippleState[jss::accounts][1u].isString() ||
            (jvRippleState[jss::accounts][0u].asString() ==
             jvRippleState[jss::accounts][1u].asString()))
        {
            jvResult[jss::error] = "malformedRequest";
        }
        else
        {
            auto const id1 = parseBase58<AccountID>(
                jvRippleState[jss::accounts][0u].asString());
            auto const id2 = parseBase58<AccountID>(
                jvRippleState[jss::accounts][1u].asString());
            if (!id1 || !id2)
            {
                jvResult[jss::error] = "malformedAddress";
            }
            else if (!to_currency(
                         uCurrency, jvRippleState[jss::currency].asString()))
            {
                jvResult[jss::error] = "malformedCurrency";
            }
            else
            {
                uNodeIndex = keylet::line(*id1, *id2, uCurrency).key;
            }
        }
    }
    else if (context.params.isMember(jss::ticket))
    {
        expectedType = ltTICKET;
        if (!context.params[jss::ticket].isObject())
        {
            if (!uNodeIndex.parseHex(context.params[jss::ticket].asString()))
            {
                uNodeIndex = beast::zero;
                jvResult[jss::error] = "malformedRequest";
            }
        }
        else if (
            !context.params[jss::ticket].isMember(jss::account) ||
            !context.params[jss::ticket].isMember(jss::ticket_seq) ||
            !context.params[jss::ticket][jss::ticket_seq].isIntegral())
        {
            jvResult[jss::error] = "malformedRequest";
        }
        else
        {
            auto const id = parseBase58<AccountID>(
                context.params[jss::ticket][jss::account].asString());
            if (!id)
                jvResult[jss::error] = "malformedAddress";
            else
                uNodeIndex = getTicketIndex(
                    *id, context.params[jss::ticket][jss::ticket_seq].asUInt());
        }
    }
    else if (context.params.isMember(jss::nft_page))
    {
        expectedType = ltNFTOKEN_PAGE;

        if (context.params[jss::nft_page].isString())
        {
            if (!uNodeIndex.parseHex(context.params[jss::nft_page].asString()))
            {
                uNodeIndex = beast::zero;
                jvResult[jss::error] = "malformedRequest";
            }
        }
        else
        {
            jvResult[jss::error] = "malformedRequest";
        }
    }
    else
    {
        if (context.params.isMember("params") &&
            context.params["params"].isArray() &&
            context.params["params"].size() == 1 &&
            context.params["params"][0u].isString())
        {
            if (!uNodeIndex.parseHex(context.params["params"][0u].asString()))
            {
                uNodeIndex = beast::zero;
                jvResult[jss::error] = "malformedRequest";
            }
        }
        else
            jvResult[jss::error] = "unknownOption";
    }

    if (uNodeIndex.isNonZero())
    {
        auto const sleNode = lpLedger->read(keylet::unchecked(uNodeIndex));
        if (context.params.isMember(jss::binary))
            bNodeBinary = context.params[jss::binary].asBool();

        if (!sleNode)
        {
            // Not found.
            jvResult[jss::error] = "entryNotFound";
        }
        else if (
            (expectedType != ltANY) && (expectedType != sleNode->getType()))
        {
            jvResult[jss::error] = "unexpectedLedgerType";
        }
        else if (bNodeBinary)
        {
            Serializer s;

            sleNode->add(s);

            jvResult[jss::node_binary] = strHex(s.peekData());
            jvResult[jss::index] = to_string(uNodeIndex);
        }
        else
        {
            jvResult[jss::node] = sleNode->getJson(JsonOptions::none);
            jvResult[jss::index] = to_string(uNodeIndex);
        }
    }

    return jvResult;
}

std::pair<org::xrpl::rpc::v1::GetLedgerEntryResponse, grpc::Status>
doLedgerEntryGrpc(
    RPC::GRPCContext<org::xrpl::rpc::v1::GetLedgerEntryRequest>& context)
{
    org::xrpl::rpc::v1::GetLedgerEntryRequest& request = context.params;
    org::xrpl::rpc::v1::GetLedgerEntryResponse response;
    grpc::Status status = grpc::Status::OK;

    std::shared_ptr<ReadView const> ledger;
    if (auto status = RPC::ledgerFromRequest(ledger, context))
    {
        grpc::Status errorStatus;
        if (status.toErrorCode() == rpcINVALID_PARAMS)
        {
            errorStatus = grpc::Status(
                grpc::StatusCode::INVALID_ARGUMENT, status.message());
        }
        else
        {
            errorStatus =
                grpc::Status(grpc::StatusCode::NOT_FOUND, status.message());
        }
        return {response, errorStatus};
    }

    auto key = uint256::fromVoidChecked(request.key());
    if (!key)
    {
        grpc::Status errorStatus{
            grpc::StatusCode::INVALID_ARGUMENT, "index malformed"};
        return {response, errorStatus};
    }

    auto const sleNode = ledger->read(keylet::unchecked(*key));
    if (!sleNode)
    {
        grpc::Status errorStatus{
            grpc::StatusCode::NOT_FOUND, "object not found"};
        return {response, errorStatus};
    }
    else
    {
        Serializer s;
        sleNode->add(s);

        auto& stateObject = *response.mutable_ledger_object();
        stateObject.set_data(s.peekData().data(), s.getLength());
        stateObject.set_key(request.key());
        *(response.mutable_ledger()) = request.ledger();
        return {response, status};
    }
}
}  // namespace ripple
