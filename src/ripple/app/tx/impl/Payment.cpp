//------------------------------------------------------------------------------
/*
    This file is part of rippled: https://github.com/ripple/rippled
    Copyright (c) 2012, 2013 Ripple Labs Inc.

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

#include <ripple/app/tx/impl/Payment.h>
#include <ripple/basics/Log.h>
#include <ripple/protocol/Feature.h>
#include <ripple/protocol/TxFlags.h>
#include <ripple/protocol/jss.h>
#include <ripple/protocol/st.h>

namespace ripple {

TxConsequences
Payment::makeTxConsequences(PreflightContext const& ctx)
{
    auto calculateMaxXRPSpend = [](STTx const& tx) -> XRPAmount {
        STAmount const maxAmount =
            tx.isFieldPresent(sfSendMax) ? tx[sfSendMax] : tx[sfAmount];

        // If there's no sfSendMax in XRP, and the sfAmount isn't
        // in XRP, then the transaction does not spend XRP.
        return maxAmount.xrp();
    };

    return TxConsequences{ctx.tx, calculateMaxXRPSpend(ctx.tx)};
}

NotTEC
Payment::preflight(PreflightContext const& ctx)
{
    if (auto const ret = preflight1(ctx); !isTesSuccess(ret))
        return ret;

    auto& tx = ctx.tx;
    auto& j = ctx.j;

    std::uint32_t const uTxFlags = tx.getFlags();

    if (uTxFlags & tfPaymentMask)
    {
        JLOG(j.trace()) << "Malformed transaction: "
                        << "Invalid flags set.";
        return temINVALID_FLAG;
    }

    bool const partialPaymentAllowed = uTxFlags & tfPartialPayment;
    bool const limitQuality = uTxFlags & tfLimitQuality;
    bool const defaultPathsAllowed = !(uTxFlags & tfNoRippleDirect);
    //    bool const bPaths = tx.isFieldPresent(sfPaths);
    bool const bMax = tx.isFieldPresent(sfSendMax);

    STAmount const saDstAmount(tx.getFieldAmount(sfAmount));

    STAmount maxSourceAmount;
    auto const account = tx.getAccountID(sfAccount);

    if (bMax)
        maxSourceAmount = tx.getFieldAmount(sfSendMax);
    else  // if (saDstAmount.native())
        maxSourceAmount = saDstAmount;

    auto const& uSrcCurrency = maxSourceAmount.getCurrency();
    auto const& uDstCurrency = saDstAmount.getCurrency();

    // isZero() is XRP.  FIX!
    //    bool const bXRPDirect = uSrcCurrency.isZero() &&
    //    uDstCurrency.isZero();

    if (!isLegalNet(saDstAmount) || !isLegalNet(maxSourceAmount))
        return temBAD_AMOUNT;

    auto const uDstAccountID = tx.getAccountID(sfDestination);

    if (!uDstAccountID)
    {
        JLOG(j.trace()) << "Malformed transaction: "
                        << "Payment destination account not specified.";
        return temDST_NEEDED;
    }
    if (bMax && maxSourceAmount <= beast::zero)
    {
        JLOG(j.trace()) << "Malformed transaction: "
                        << "bad max amount: " << maxSourceAmount.getFullText();
        return temBAD_AMOUNT;
    }
    if (saDstAmount <= beast::zero)
    {
        JLOG(j.trace()) << "Malformed transaction: "
                        << "bad dst amount: " << saDstAmount.getFullText();
        return temBAD_AMOUNT;
    }
    if (badCurrency() == uSrcCurrency || badCurrency() == uDstCurrency)
    {
        JLOG(j.trace()) << "Malformed transaction: "
                        << "Bad currency.";
        return temBAD_CURRENCY;
    }
    if (account == uDstAccountID && uSrcCurrency == uDstCurrency)
    {
        // You're signing yourself a payment.
        JLOG(j.trace()) << "Malformed transaction: "
                        << "Redundant payment from " << to_string(account)
                        << " to self without path for "
                        << to_string(uDstCurrency);
        return temREDUNDANT;
    }
    if (bMax)
    {
        // Consistent but redundant transaction.
        JLOG(j.trace()) << "Malformed transaction: "
                        << "SendMax specified for XRP to XRP.";
        return temBAD_SEND_XRP_MAX;
    }
    if (partialPaymentAllowed)
    {
        // Consistent but redundant transaction.
        JLOG(j.trace()) << "Malformed transaction: "
                        << "Partial payment specified for XRP to XRP.";
        return temBAD_SEND_XRP_PARTIAL;
    }
    if (limitQuality)
    {
        // Consistent but redundant transaction.
        JLOG(j.trace()) << "Malformed transaction: "
                        << "Limit quality specified for XRP to XRP.";
        return temBAD_SEND_XRP_LIMIT;
    }
    if (!defaultPathsAllowed)
    {
        // Consistent but redundant transaction.
        JLOG(j.trace()) << "Malformed transaction: "
                        << "No ripple direct specified for XRP to XRP.";
        return temBAD_SEND_XRP_NO_DIRECT;
    }

    auto const deliverMin = tx[~sfDeliverMin];
    if (deliverMin)
    {
        if (!partialPaymentAllowed)
        {
            JLOG(j.trace()) << "Malformed transaction: Partial payment not "
                               "specified for "
                            << jss::DeliverMin.c_str() << ".";
            return temBAD_AMOUNT;
        }

        auto const dMin = *deliverMin;
        if (!isLegalNet(dMin) || dMin <= beast::zero)
        {
            JLOG(j.trace())
                << "Malformed transaction: Invalid " << jss::DeliverMin.c_str()
                << " amount. " << dMin.getFullText();
            return temBAD_AMOUNT;
        }
        if (dMin.issue() != saDstAmount.issue())
        {
            JLOG(j.trace())
                << "Malformed transaction: Dst issue differs "
                   "from "
                << jss::DeliverMin.c_str() << ". " << dMin.getFullText();
            return temBAD_AMOUNT;
        }
        if (dMin > saDstAmount)
        {
            JLOG(j.trace())
                << "Malformed transaction: Dst amount less than "
                << jss::DeliverMin.c_str() << ". " << dMin.getFullText();
            return temBAD_AMOUNT;
        }
    }

    return preflight2(ctx);
}

TER
Payment::preclaim(PreclaimContext const& ctx)
{
    // Ripple if source or destination is non-native or if there are paths.
    std::uint32_t const uTxFlags = ctx.tx.getFlags();
    bool const partialPaymentAllowed = uTxFlags & tfPartialPayment;
    //    auto const paths = ctx.tx.isFieldPresent(sfPaths);
    auto const sendMax = ctx.tx[~sfSendMax];

    AccountID const uDstAccountID(ctx.tx[sfDestination]);
    STAmount const saDstAmount(ctx.tx[sfAmount]);

    auto const k = keylet::account(uDstAccountID);
    auto const sleDst = ctx.view.read(k);

    if (!sleDst)
    {
        if (ctx.view.open() && partialPaymentAllowed)
        {
            // You cannot fund an account with a partial payment.
            // Make retry work smaller, by rejecting this.
            JLOG(ctx.j.trace()) << "Delay transaction: Partial payment not "
                                   "allowed to create account.";

            // Another transaction could create the account and then this
            // transaction would succeed.
            return telNO_DST_PARTIAL;
        }
        else if (saDstAmount < STAmount(ctx.view.fees().accountReserve(0)))
        {
            // accountReserve is the minimum amount that an account can have.
            // Reserve is not scaled by load.
            JLOG(ctx.j.trace())
                << "Delay transaction: Destination account does not exist. "
                << "Insufficent payment to create account.";

            // TODO: dedupe
            // Another transaction could create the account and then this
            // transaction would succeed.
            return tecNO_DST_INSUF_XRP;
        }
    }
    else if (
        (sleDst->getFlags() & lsfRequireDestTag) &&
        !ctx.tx.isFieldPresent(sfDestinationTag))
    {
        // The tag is basically account-specific information we don't
        // understand, but we can require someone to fill it in.

        // We didn't make this test for a newly-formed account because there's
        // no way for this field to be set.
        JLOG(ctx.j.trace())
            << "Malformed transaction: DestinationTag required.";

        return tecDST_TAG_NEEDED;
    }

    return tesSUCCESS;
}

TER
Payment::doApply()
{
    //    auto const deliverMin = ctx_.tx[~sfDeliverMin];

    // Ripple if source or destination is non-native or if there are paths.
    //    std::uint32_t const uTxFlags = ctx_.tx.getFlags();
    //    bool const partialPaymentAllowed = uTxFlags & tfPartialPayment;
    //    bool const limitQuality = uTxFlags & tfLimitQuality;
    //    bool const defaultPathsAllowed = !(uTxFlags & tfNoRippleDirect);
    //    auto const paths = ctx_.tx.isFieldPresent(sfPaths);
    //    auto const sendMax = ctx_.tx[~sfSendMax];

    AccountID const uDstAccountID(ctx_.tx.getAccountID(sfDestination));
    STAmount const saDstAmount(ctx_.tx.getFieldAmount(sfAmount));
    //    STAmount maxSourceAmount;
    //    if (sendMax)
    //        maxSourceAmount = *sendMax;
    //    else //if (saDstAmount.native())
    //        maxSourceAmount = saDstAmount;
    //    else
    //        maxSourceAmount = STAmount(
    //            {saDstAmount.getCurrency(), account_},
    //            saDstAmount.mantissa(),
    //            saDstAmount.exponent(),
    //            saDstAmount < beast::zero);

    JLOG(j_.trace())
        //        << "maxSourceAmount=" << maxSourceAmount.getFullText()
        << " saDstAmount=" << saDstAmount.getFullText();

    // Open a ledger for editing.
    auto const k = keylet::account(uDstAccountID);
    SLE::pointer sleDst = view().peek(k);

    if (!sleDst)
    {
        std::uint32_t const seqno{
            view().rules().enabled(featureDeletableAccounts) ? view().seq()
                                                             : 1};

        // Create the account.
        sleDst = std::make_shared<SLE>(k);
        sleDst->setAccountID(sfAccount, uDstAccountID);
        sleDst->setFieldU32(sfSequence, seqno);

        view().insert(sleDst);
    }
    else
    {
        // Tell the engine that we are intending to change the destination
        // account.  The source account gets always charged a fee so it's always
        // marked as modified.
        view().update(sleDst);
    }

    //    assert(saDstAmount.native());

    // Direct XRP payment.

    auto const sleSrc = view().peek(keylet::account(account_));
    if (!sleSrc)
        return tefINTERNAL;

    // uOwnerCount is the number of entries in this ledger for this
    // account that require a reserve.
    auto const uOwnerCount = sleSrc->getFieldU32(sfOwnerCount);

    // This is the total reserve in drops.
    auto const reserve = view().fees().accountReserve(uOwnerCount);

    // mPriorBalance is the balance on the sending account BEFORE the
    // fees were charged. We want to make sure we have enough reserve
    // to send. Allow final spend to use reserve for fee.
    auto const mmm = std::max(reserve, ctx_.tx.getFieldAmount(sfFee).xrp());

    if (mPriorBalance < saDstAmount.xrp() + mmm)
    {
        // Vote no. However the transaction might succeed, if applied in
        // a different order.
        JLOG(j_.trace()) << "Delay transaction: Insufficient funds: "
                         << " " << to_string(mPriorBalance) << " / "
                         << to_string(saDstAmount.xrp() + mmm) << " ("
                         << to_string(reserve) << ")";

        return tecUNFUNDED_PAYMENT;
    }

    // Do the arithmetic for the transfer and make the ledger change.
    sleSrc->setFieldAmount(sfBalance, mSourceBalance - saDstAmount);
    sleDst->setFieldAmount(
        sfBalance, sleDst->getFieldAmount(sfBalance) + saDstAmount);

    // Re-arm the password change fee if we can and need to.
    if ((sleDst->getFlags() & lsfPasswordSpent))
        sleDst->clearFlag(lsfPasswordSpent);

    return tesSUCCESS;
}

}  // namespace ripple
