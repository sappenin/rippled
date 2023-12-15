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

#include <ripple/protocol/Feature.h>
#include <test/jtx.h>
#include <test/jtx/Account.h>
#include <ripple/json/json_value.h>
#include <ripple/protocol/digest.h>
#include <ripple/basics/strHex.h>
#include <ripple/protocol/SField.h>

namespace ripple {

    class Ballot_test : public beast::unit_test::suite {

        ///////////////////////
        // Tests to write
        // TODO: Membership NFT token & taxon.
        ///////////////////////

        void
        testWhenCreateDisabled(FeatureBitset features) {
            testcase("CreateBallot featureBallotsV1 Disabled");

            using namespace test::jtx;

            // If the BallotsV1 amendment is not enabled, you should not be able to create or delete ballots.
            Env env{*this, features - featureBallotsV1};
            Account const alice{"alice"};
            env.fund(XRP(5000), alice);
            env.close();

            BEAST_EXPECT(env.ownerCount(alice) == 0);
            BEAST_EXPECT(accountBalance(env, alice) == "5000000000");

            // Try to create a ballot when the feature is disabled.
            env(ballot::create(alice), ter(temDISABLED));
            env.close();

            BEAST_EXPECT(env.ownerCount(alice) == 0);
        }

        void
        testCreateBallotValidationInitialVotes(FeatureBitset features) {
            testcase("CreateBallot validation for InitialVotes");

            using namespace test::jtx;

            // If the BallotsV1 amendment is not enabled, you should not be able to create or delete ballots.
            Env env{*this, features};
            Account const alice{"alice"};
            env.fund(XRP(5000), alice);
            env.close();

            BEAST_EXPECT(env.ownerCount(alice) == 0);
            BEAST_EXPECT(accountBalance(env, alice) == "5000000000");

            // Try to create with an invalid flag.
            env(ballot::create(alice), txflags(0x00000001), ter(temINVALID_FLAG));
            env.close();

            ////////////////
            // InitialVotes
            ////////////////

            // Try to create without setting sfInitialVotes
            {
                Json::Value jv = ballot::create(alice);
                jv.removeMember(sfInitialVotes.jsonName);
                env(jv, ter(temMALFORMED));
                env.close();
                BEAST_EXPECT(env.ownerCount(alice) == 0);
            }

            // Try to create with 0 sfInitialVotes
            {
                Json::Value jv = ballot::create(alice);
                jv.removeMember(sfInitialVotes.jsonName);
                jv[sfInitialVotes.jsonName] = (std::uint32_t) 0;
                env(jv, ter(temMALFORMED));
                env.close();
                BEAST_EXPECT(env.ownerCount(alice) == 0);
            }
        }

        void
        testCreateBallotValidationBallotChoiceIDs(FeatureBitset features) {
            testcase("CreateBallot Validation for BallotChoiceIDs");

            using namespace test::jtx;

            // If the BallotsV1 amendment is not enabled, you should not be able to create or delete ballots.
            Env env{*this, features};
            Account const alice{"alice"};
            env.fund(XRP(5000), alice);
            env.close();

            BEAST_EXPECT(env.ownerCount(alice) == 0);
            BEAST_EXPECT(accountBalance(env, alice) == "5000000000");

            // Try to create with an invalid flag.
            env(ballot::create(alice), txflags(0x00000001), ter(temINVALID_FLAG));
            env.close();

            //////////////////
            // BallotChoiceIDs
            //////////////////

            // Try to create with missing BallotChoiceIDs
            {
                Json::Value jv = ballot::create(alice);
                jv.removeMember(sfBallotChoiceIDs.jsonName);
                env(jv, ter(temMALFORMED));
                env.close();
                BEAST_EXPECT(env.ownerCount(alice) == 0);
            }

            // Try to create with empty BallotChoiceIDs
            {
                Json::Value jv = ballot::create(alice);
                jv.removeMember(sfBallotChoiceIDs.jsonName);
                Json::Value emptyBallotChoiceIDs(Json::arrayValue);
                jv[sfBallotChoiceIDs.jsonName] = emptyBallotChoiceIDs;
                env(jv, ter(temMALFORMED));
                env.close();
                BEAST_EXPECT(env.ownerCount(alice) == 0);
            }

            // Try to create with too many BallotChoiceIDs
            {
                Json::Value jv = ballot::create(alice);
                jv.removeMember(sfBallotChoiceIDs.jsonName);
                Json::Value tooBigChoiceIDs(Json::arrayValue);
                uint256 ballotChoiceID = sha512Half(0); // <-- arbitrary value for hashing
                for (int i = 0; i < (maxBallotChoiceIDsLength + 1); i++) {
                    tooBigChoiceIDs.append(to_string(ballotChoiceID));
                }
                jv[sfBallotChoiceIDs.jsonName] = tooBigChoiceIDs;
                env(jv, ter(temMALFORMED));
                env.close();
                BEAST_EXPECT(env.ownerCount(alice) == 0);
            }
        }

        void
        testCreateBallotValidationBallotDocumentURI(FeatureBitset features) {
            testcase("CreateBallot Validation for BallotDocumentURI");

            using namespace test::jtx;

            // If the BallotsV1 amendment is not enabled, you should not be able to create or delete ballots.
            Env env{*this, features};
            Account const alice{"alice"};
            env.fund(XRP(5000), alice);
            env.close();

            BEAST_EXPECT(env.ownerCount(alice) == 0);
            BEAST_EXPECT(accountBalance(env, alice) == "5000000000");

            // Try to create with an invalid flag.
            env(ballot::create(alice), txflags(0x00000001), ter(temINVALID_FLAG));
            env.close();

            ////////////////
            // BallotDocumentURI
            ////////////////

            // Try to Use an empty URI
            {
                Json::Value jv = ballot::create(alice);
                jv[sfBallotDocumentURI.jsonName] = strHex(ballot::empty);
                env(jv, ter(temMALFORMED));
                env.close();
                BEAST_EXPECT(env.ownerCount(alice) == 0);
            }

            // Try to a DocumentURI that's too long
            {
                Json::Value jv = ballot::create(alice);
                jv[sfBallotDocumentURI.jsonName] = strHex(ballot::DOCUMENT_URI_TOO_LONG);
                env(jv, ter(temMALFORMED));
                env.close();
                BEAST_EXPECT(env.ownerCount(alice) == 0);
            }
        }

        void
        testCreateBallotValidationBallotDocumentHash(FeatureBitset features) {
            testcase("CreateBallot Validation for BallotDocumentHash");

            using namespace test::jtx;

            // If the BallotsV1 amendment is not enabled, you should not be able to create or delete ballots.
            Env env{*this, features};
            Account const alice{"alice"};
            env.fund(XRP(5000), alice);
            env.close();

            BEAST_EXPECT(env.ownerCount(alice) == 0);
            BEAST_EXPECT(accountBalance(env, alice) == "5000000000");

            // Try to create with an invalid flag.
            env(ballot::create(alice), txflags(0x00000001), ter(temINVALID_FLAG));
            env.close();

            ////////////////
            // BallotDocumentHash
            ////////////////

            // Try with a DocumentURI, but missing DocumentHash
            {
                Json::Value jv = ballot::create(alice);
                jv.removeMember(sfBallotDocumentHash.jsonName);
                env(jv, ter(temMALFORMED));
                env.close();
                BEAST_EXPECT(env.ownerCount(alice) == 0);
            }

            // Try with a missing DocumentURI, but DocumentHash present and n
            {
                Json::Value jv = ballot::create(alice);
                jv.removeMember(sfBallotDocumentURI.jsonName);
                env(jv, ter(tesSUCCESS));
                env.close();
                // std::cout << "OwnerCount: " << env.ownerCount(alice) << std::endl;
                BEAST_EXPECT(env.ownerCount(alice) == 1);
            }

        }

        void
        testCreateBallotValidationOpenCloseTimes(FeatureBitset features) {
            testcase("CreateBallot Validation for Open/Close Times");

            using namespace test::jtx;

            // If the BallotsV1 amendment is not enabled, you should not be able to create or delete ballots.
            Env env{*this, features};
            Account const alice{"alice"};
            env.fund(XRP(5000), alice);
            env.close();

            BEAST_EXPECT(env.ownerCount(alice) == 0);
            BEAST_EXPECT(accountBalance(env, alice) == "5000000000");

            // Try to create with an invalid flag.
            env(ballot::create(alice), txflags(0x00000001), ter(temINVALID_FLAG));
            env.close();

            ////////////////
            // Open/Close Times
            ////////////////

            // Close Time is before the open time.
            {
                Json::Value jv = ballot::create(alice);
                jv[sfOpenTime.jsonName] = (uint32_t) 5;
                jv[sfCloseTime.jsonName] = (uint32_t) 4;
                env(jv, ter(temMALFORMED));
                env.close();
                BEAST_EXPECT(env.ownerCount(alice) == 0);
            }

            // Close Time is equal to the open time.
            {
                Json::Value jv = ballot::create(alice);
                jv[sfOpenTime.jsonName] = (uint32_t) 5;
                jv[sfCloseTime.jsonName] = (uint32_t) 5;
                env(jv, ter(tesSUCCESS));
                env.close();
                BEAST_EXPECT(env.ownerCount(alice) == 1);
            }

        }

        void
        testCreateBallotValidationMembershipNFT(FeatureBitset features) {
            testcase("CreateBallot Validation for Membership NFT");

            using namespace test::jtx;

            // If the BallotsV1 amendment is not enabled, you should not be able to create or delete ballots.
            Env env{*this, features};
            Account const alice{"alice"};
            Account const nft_issuer{"nft_issuer"};
            env.fund(XRP(5000), alice);
            env.close();

            BEAST_EXPECT(env.ownerCount(alice) == 0);
            BEAST_EXPECT(accountBalance(env, alice) == "5000000000");

            // Try to create with an invalid flag.
            env(ballot::create(alice), txflags(0x00000001), ter(temINVALID_FLAG));
            env.close();

            ////////////////
            // Open/Close Times
            ////////////////

            // Don't allow a Taxon without an issuer.
            {
                Json::Value jv = ballot::create(alice);
                jv.removeMember(sfMembershipNFTIssuer.jsonName);
                jv[sfMembershipNFTTaxon.jsonName] = (uint32_t) 1;
                env(jv, ter(temMALFORMED));
                env.close();
                BEAST_EXPECT(env.ownerCount(alice) == 0);
            }

            // Allow an issuer with no taxon
            {
                Json::Value jv = ballot::create(alice);
                jv[sfMembershipNFTIssuer.jsonName] = nft_issuer.human();
                jv.removeMember(sfMembershipNFTTaxon.jsonName);
                env(jv, ter(tesSUCCESS));
                env.close();
                BEAST_EXPECT(env.ownerCount(alice) == 1);
            }

            // Allow both an issuer and a taxon
            {
                Json::Value jv = ballot::create(alice);
                jv[sfMembershipNFTIssuer.jsonName] = nft_issuer.human();
                jv[sfMembershipNFTTaxon.jsonName] = (uint32_t) 1;
                env(jv, ter(tesSUCCESS));
                env.close();
                BEAST_EXPECT(env.ownerCount(alice) == 2);
            }

        }

        void
        testCreate(FeatureBitset features) {
            testcase("Create Ballot");

            using namespace test::jtx;

            // If the Ballot amendment IS enabled, you should be able to create Ballots
            Env env{*this, features};
            Account const alice("alice");  // issuer
            env.fund(XRP(5000), alice);
            env.close();

            BEAST_EXPECT(env.ownerCount(alice) == 0);
            BEAST_EXPECT(accountBalance(env, alice) == "5000000000");

            env(ballot::create(alice), ter(tesSUCCESS));
            env.close();

            BEAST_EXPECT(env.ownerCount(alice) == 1);
        }

    public:

        void
        run()

        override {
            using namespace test::jtx;
            FeatureBitset const all{supported_amendments()};

            // BallotCreate
            testWhenCreateDisabled(all);
            testCreateBallotValidationInitialVotes(all);
            testCreateBallotValidationBallotChoiceIDs(all);
            testCreateBallotValidationBallotDocumentHash(all);
            testCreateBallotValidationBallotDocumentURI(all);
            testCreateBallotValidationOpenCloseTimes(all);
            testCreateBallotValidationMembershipNFT(all);
            testCreate(all);

            // BallotDelete
            // TODO Test BallotDelete

            // BallotVote
            // TODO Test BallotVote
        }
    };

    BEAST_DEFINE_TESTSUITE_PRIO(Ballot, tx, ripple, 2);

}  // namespace ripple