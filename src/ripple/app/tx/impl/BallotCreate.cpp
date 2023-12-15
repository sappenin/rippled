//------------------------------------------------------------------------------
/*
  This file is part of rippled: https://github.com/ripple/rippled
  Copyright (c) 2023 Ripple Labs Inc.

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

#include <ripple/app/tx/impl/BallotCreate.h>
#include <ripple/ledger/View.h>
#include <ripple/protocol/Feature.h>
#include <ripple/protocol/TxFlags.h>
#include <ripple/protocol/Indexes.h>
#include <ripple/protocol/st.h>
#include <ripple/protocol/AccountID.h>

namespace ripple {

    NotTEC
    BallotCreate::preflight(PreflightContext const &ctx) {
        if (!ctx.rules.enabled(featureBallotsV1)) {
            JLOG(ctx.j.debug()) << "BallotCreate: feature disabled.";
            return temDISABLED;
        }

        if (ctx.tx.getFlags() & tfUniversalMask) {
            JLOG(ctx.j.debug()) << "BallotCreate: invalid flags.";
            return temINVALID_FLAG;
        }

        if (!ctx.tx.isFieldPresent(sfInitialVotes) || ctx.tx[sfInitialVotes] < 1) {
            JLOG(ctx.j.debug()) << "BallotCreate: InitialVotes must be greater than 0.";
            return temMALFORMED;
        }

        if (!ctx.tx.isFieldPresent(sfBallotChoiceIDs)) {
            JLOG(ctx.j.debug()) << "BallotCreate: BallotChoiceIDs must be present.";
            return temMALFORMED;
        } else {
            if (ctx.tx[sfBallotChoiceIDs].empty()) {
                JLOG(ctx.j.debug()) << "BallotCreate: BallotChoiceIDs must not be empty.";
                return temMALFORMED;
            }

            if (ctx.tx[sfBallotChoiceIDs].size() > maxBallotChoiceIDsLength) {
                JLOG(ctx.j.debug()) << "BallotCreate: BallotChoiceIDs must not be greater than "
                                    << maxBallotChoiceIDsLength;
                return temMALFORMED;
            }
        }

        if (auto const ret = preflight1(ctx); !isTesSuccess(ret))
            return ret;

        if (ctx.tx.isFieldPresent(sfBallotDocumentURI)) {
            // If there's a BallotDocumentURI, there must be a Hash (but not vice-versa, it's fine to specify a hash
            // but not a URI, e.g., if the URI isn't public). If there's a hash, it must have a non-zero length.
            if (!ctx.tx.isFieldPresent(sfBallotDocumentHash)) {
                JLOG(ctx.j.debug())
                    << "BallotCreate: BallotDocumentHash must be present if BallotDocumentURI is specified.";
                return temMALFORMED;
            }

            if (ctx.tx[sfBallotDocumentURI].empty()) { // <-- URI can't be empty size.
                JLOG(ctx.j.debug()) << "BallotCreate: BallotDocumentURI must not be empty if specified.";
                return temMALFORMED;
            }

            auto isTooLong = [&](auto const &sField, std::size_t length) -> bool {
                if (auto field = ctx.tx[~sField])
                    return field->length() > length;
                return false;
            };

            if (isTooLong(sfBallotDocumentURI, maxBallotDocumentUriLength)) {
                JLOG(ctx.j.debug()) << "BallotCreate: BallotDocumentYURI was too long. Length="
                                    << ctx.tx[sfBallotDocumentURI].length();
                return temMALFORMED;
            }
        }

        // The close time must be after the open time if both are present.
        if (ctx.tx.isFieldPresent(sfOpenTime) && ctx.tx.isFieldPresent(sfCloseTime)) {
            if (ctx.tx[sfOpenTime] > ctx.tx[sfCloseTime]) {
                JLOG(ctx.j.debug()) << "BallotCreate: CloseTime must be after OpenTime.";
                return temMALFORMED;
            }
        }

        if(ctx.tx.isFieldPresent(sfMembershipNFTTaxon)){
            if(!ctx.tx.isFieldPresent(sfMembershipNFTIssuer)){
                JLOG(ctx.j.debug()) <<
                "BallotCreate: Ballots with a MembershipNFTTaxon must specify a MembershipNFTIssuer.";
                return temMALFORMED;
            }
        }

        return preflight2(ctx);
    }

    TER
    BallotCreate::preclaim(PreclaimContext const &ctx) {
        return tesSUCCESS;
    }

    TER
    BallotCreate::doApply() {
        auto const acct = view().peek(keylet::account(account_));
        if (!acct)
            return tecINTERNAL;

        if (mPriorBalance < view().fees().accountReserve((*acct)[sfOwnerCount] + 1))
            return tecINSUFFICIENT_RESERVE;

        auto const ballotID = keylet::ballot(account_, ctx_.tx.getSeqProxy().value());

        // create the Ballot
        {
            auto const ownerNode = view().dirInsert(
                    keylet::ownerDir(account_),
                    ballotID,
                    describeOwnerDir(account_)
            );

            if (!ownerNode)
                return tecDIR_FULL;

            auto ballot = std::make_shared<SLE>(ballotID);
            (*ballot)[sfFlags] = ctx_.tx.getFlags() & ~tfUniversal;
            (*ballot)[sfOwnerNode] = *ownerNode;

            if (auto const initialVotes = ctx_.tx[~sfInitialVotes])
                (*ballot)[sfInitialVotes] = *initialVotes;
//            if (auto const openTime = ctx_.tx[~sfOpenTime])
//                (*ballot)[sfOpenTime] = *openTime;
//            if (auto const closeTime = ctx_.tx[~sfCloseTime])
//                (*ballot)[sfOpenTime] = *closeTime;
//            if (auto const ballotDocumentUri = ctx_.tx[~sfBallotDocumentURI])
//                (*ballot)[sfBallotDocumentURI] = *ballotDocumentUri;
//            if (auto const ballotDocumentHash = ctx_.tx[~sfBallotDocumentHash])
//                (*ballot)[sfBallotDocumentHash] = *ballotDocumentHash;
            if (auto const ballotChoiceIDs = ctx_.tx[~sfBallotChoiceIDs])
                (*ballot)[sfBallotChoiceIDs] = *ballotChoiceIDs;

            // TODO: LockedAmount?
            // TODO: Membership NFT Issuer & Taxon

            // TODO:


            //            if (auto const scale = ctx_.tx[~sfAssetScale])
//                (*cftIssuance)[sfAssetScale] = *scale;
//
//            if (auto const fee = ctx_.tx[~sfTransferFee])
//                (*cftIssuance)[sfTransferFee] = *fee;
//
//            if (auto const metadata = ctx_.tx[~sfCFTokenMetadata])
//                (*cftIssuance)[sfCFTokenMetadata] = *metadata;
//
            view().insert(ballot);
        }

        // Update owner count.
        adjustOwnerCount(view(), acct, 1, j_);

        return tesSUCCESS;
    }

}  // namespace ripple