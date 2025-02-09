import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import {
    PublicKey,
    Keypair,
    SystemProgram,
    LAMPORTS_PER_SOL,
    Connection,
    Commitment,
    Transaction
} from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, createAssociatedTokenAccount, getAssociatedTokenAddress, getAccount, getOrCreateAssociatedTokenAccount } from '@solana/spl-token';
import { ContributionTracker } from '../target/types/contribution_tracker';
import {
    createTestMint,
    setupTokenVaults,
    EPOCH_DURATION,
    getCurrentPeriodPda
} from './utils';
import { expect } from 'chai';

const sleep = (ms: number): Promise<void> => {
    return new Promise(resolve => setTimeout(resolve, ms));
};

const waitForTransaction = async (
    connection: Connection,
    signature: string,
    maxRetries = 30,
    retryInterval = 1000
): Promise<void> => {
    let retries = 0;

    while (retries < maxRetries) {
        const status = await connection.getSignatureStatus(signature);

        if (status.value?.confirmationStatus === 'confirmed') {
            return;
        }

        await sleep(retryInterval);
        retries++;
    }

    throw new Error(`Transaction ${signature} was not confirmed after ${maxRetries} attempts`);
};

describe('contribution-tracker', () => {
    const commitment: Commitment = 'confirmed';
    const connection = new Connection('http://localhost:8899', commitment);

    const provider = new anchor.AnchorProvider(
        connection,
        anchor.Wallet.local(),
        {
            commitment,
            preflightCommitment: commitment,
            skipPreflight: false
        }
    );

    anchor.setProvider(provider);

    const program = anchor.workspace.ContributionTracker as Program<ContributionTracker>;

    let mint: PublicKey;
    let rewardVault: PublicKey;
    let reserveVault: PublicKey;
    let contributor: Keypair;
    let trackerPda: PublicKey;
    let periodPda: PublicKey;
    let lastContributionPda: PublicKey;

    before(async () => {
        try {
            console.log("Starting test setup...");

            // Initialize contributor
            contributor = Keypair.generate();
            console.log("Created contributor keypair");

            // Airdrop
            const airdropSig = await connection.requestAirdrop(
                contributor.publicKey,
                2 * LAMPORTS_PER_SOL
            );
            await provider.connection.confirmTransaction(airdropSig);
            console.log("Airdrop completed");

            // Get PDAs
            [trackerPda] = PublicKey.findProgramAddressSync(
                [Buffer.from("contribution_tracker")],
                program.programId
            );

            [periodPda] = PublicKey.findProgramAddressSync(
                [Buffer.from("distribution_period"), Buffer.from([0])],
                program.programId
            );
            console.log("PDAs generated");

            // Setup token infrastructure
            console.log("Creating test mint...");
            mint = await createTestMint(provider);
            console.log("Test mint created:", mint.toBase58());

            console.log("Setting up token vaults...");
            const vaults = await setupTokenVaults(provider, mint);
            rewardVault = vaults.rewardVault;
            reserveVault = vaults.reserveVault;
            console.log("Token setup complete");

        } catch (error) {
            console.error("Detailed setup error:", error);
            throw error;
        }
    });

    it("Initializes contribution tracker", async () => {
        await sleep(2000);

        try {
            console.log("Starting tracker initialization...");
            console.log("Admin pubkey:", provider.wallet.publicKey.toBase58());
            console.log("Tracker PDA:", trackerPda.toBase58());
            console.log("Period PDA:", periodPda.toBase58());

            // Verify the PDAs
            const [expectedTrackerPda] = PublicKey.findProgramAddressSync(
                [Buffer.from("contribution_tracker")],
                program.programId
            );
            console.log("Expected Tracker PDA:", expectedTrackerPda.toBase58());

            const [expectedPeriodPda] = PublicKey.findProgramAddressSync(
                [
                    Buffer.from("distribution_period"),
                    Buffer.from([0, 0, 0, 0, 0, 0, 0, 0])  // 0u64 as bytes
                ],
                program.programId
            );
            console.log("Expected Period PDA:", expectedPeriodPda.toBase58());

            // Build and send transaction
            const tx = await program.methods
                .initializeTracker(
                    new anchor.BN(3),          // period_duration
                    100,                       // minimum_points_threshold
                    new anchor.BN(10000)       // tokens_per_period
                )
                .accounts({
                    admin: provider.wallet.publicKey,
                    contributionTracker: expectedTrackerPda,
                    initialPeriod: expectedPeriodPda,
                    systemProgram: SystemProgram.programId,
                })
                .signers([provider.wallet.payer])
                .rpc();

            console.log("Transaction signature:", tx);

            // Wait for confirmation
            const latestBlockhash = await connection.getLatestBlockhash();
            await connection.confirmTransaction({
                signature: tx,
                blockhash: latestBlockhash.blockhash,
                lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
            }, 'confirmed');

            // Verify the accounts
            const tracker = await program.account.contributionTracker.fetch(
                expectedTrackerPda
            );

            expect(tracker.admin).to.eql(provider.wallet.publicKey);
            console.log("Initialization successful");

        } catch (error) {
            console.error("Detailed test error:", error);
            throw error;
        }
    });

    it("Record contribution", async () => {
        try {
            // Get the current period from the tracker
            const tracker = await program.account.contributionTracker.fetch(trackerPda);
            console.log("Current period number:", tracker.currentPeriod.toString());

            // Calculate the correct period PDA
            const [currentPeriodPda] = PublicKey.findProgramAddressSync(
                [
                    Buffer.from("distribution_period"),
                    new anchor.BN(tracker.currentPeriod).toArrayLike(Buffer, 'le', 8)
                ],
                program.programId
            );
            console.log("Current period PDA:", currentPeriodPda.toBase58());

            const [contributorPda] = PublicKey.findProgramAddressSync(
                [
                    Buffer.from("contributor"),
                    contributor.publicKey.toBuffer()
                ],
                program.programId
            );
            console.log("Contributor PDA:", contributorPda.toBase58());

            const [contributionPda] = PublicKey.findProgramAddressSync(
                [
                    Buffer.from("contribution"),
                    contributor.publicKey.toBuffer(),
                    new anchor.BN(tracker.currentPeriod).toArrayLike(Buffer, 'le', 8),
                    new anchor.BN(0).toArrayLike(Buffer, 'le', 8)
                ],
                program.programId
            );
            console.log("Contribution PDA:", contributionPda.toBase58());

            lastContributionPda = contributionPda;

            console.log("Recording contribution...");
            const tx = await program.methods
                .recordContribution(
                    { bugFix: {} },
                    { major: {} },
                    "Fixed critical memory leak"
                )
                .accounts({
                    contributor: contributor.publicKey,
                    contributionTracker: trackerPda,
                    currentPeriod: currentPeriodPda,
                    contributorAccount: contributorPda,
                    contribution: contributionPda,
                    systemProgram: SystemProgram.programId,
                })
                .signers([contributor])
                .rpc();

            console.log("Transaction signature:", tx);

            // Wait for confirmation
            await provider.connection.confirmTransaction(tx);

            // Verify the contribution
            const contribution = await program.account.contribution.fetch(
                contributionPda
            );
            console.log("Contribution recorded:", contribution);

            expect(contribution.status).to.eql({ pending: {} });
            expect(contribution.points).to.equal(7);
        } catch (error) {
            console.error("Error in record contribution:", error);
            throw error;
        }
    });

    it("Reviews contribution", async () => {
        try {
            // Get the current period from the tracker
            const tracker = await program.account.contributionTracker.fetch(trackerPda);
            console.log("Current period number:", tracker.currentPeriod.toString());

            // Calculate the correct period PDA
            const [currentPeriodPda] = PublicKey.findProgramAddressSync(
                [
                    Buffer.from("distribution_period"),
                    new anchor.BN(tracker.currentPeriod).toArrayLike(Buffer, 'le', 8)
                ],
                program.programId
            );
            console.log("Current period PDA:", currentPeriodPda.toBase58());

            const [contributorPda] = PublicKey.findProgramAddressSync(
                [
                    Buffer.from("contributor"),
                    contributor.publicKey.toBuffer()
                ],
                program.programId
            );

            console.log("Reviewing contribution:", lastContributionPda.toBase58());

            const tx = await program.methods
                .reviewContribution(true)
                .accounts({
                    admin: provider.wallet.publicKey,
                    contributionTracker: trackerPda,
                    currentPeriod: currentPeriodPda,
                    contributorAccount: contributorPda,
                    contribution: lastContributionPda,
                })
                .rpc();

            await provider.connection.confirmTransaction(tx);

            const contribution = await program.account.contribution.fetch(
                lastContributionPda
            );
            console.log("Contribution after review:", contribution);

            expect(contribution.status).to.eql({ approved: {} });
        } catch (error) {
            console.error("Error in review contribution:", error);
            // Log the detailed error information
            if (error.logs) {
                console.error("Transaction logs:", error.logs);
            }
            throw error;
        }
    });

    it("Processes period distribution", async () => {
        try {
            // Wait for period to end
            await new Promise((resolve) => setTimeout(resolve, EPOCH_DURATION * 1000));

            // Get the current period from the tracker
            const tracker = await program.account.contributionTracker.fetch(trackerPda);
            console.log("Current period number:", tracker.currentPeriod.toString());

            // Calculate the correct period PDA
            const [currentPeriodPda] = PublicKey.findProgramAddressSync(
                [
                    Buffer.from("distribution_period"),
                    new anchor.BN(tracker.currentPeriod).toArrayLike(Buffer, 'le', 8)
                ],
                program.programId
            );
            console.log("Current period PDA:", currentPeriodPda.toBase58());

            const tx = await program.methods
                .processPeriodDistribution()
                .accounts({
                    admin: provider.wallet.publicKey,
                    contributionTracker: trackerPda,
                    periodToProcess: currentPeriodPda,
                    rewardVault: rewardVault,
                    reserveVault: reserveVault,
                    rewardMint: mint,
                    tokenProgram: TOKEN_PROGRAM_ID,
                })
                .rpc();

            // Wait for confirmation
            await provider.connection.confirmTransaction(tx);
            console.log("Distribution processed. Transaction:", tx);

            // Fetch from the correct PDA
            const period = await program.account.distributionPeriod.fetch(
                currentPeriodPda
            );
            console.log("Period after distribution:", period);

            expect(period.isFinalized).to.be.true;

            // Additional verification
            const updatedTracker = await program.account.contributionTracker.fetch(trackerPda);
            console.log("Updated tracker:", updatedTracker);

            // Verify token balances if needed
            const rewardVaultBalance = await provider.connection.getTokenAccountBalance(rewardVault);
            const reserveVaultBalance = await provider.connection.getTokenAccountBalance(reserveVault);
            console.log("Reward vault balance:", rewardVaultBalance.value.uiAmount);
            console.log("Reserve vault balance:", reserveVaultBalance.value.uiAmount);

        } catch (error) {
            console.error("Error in process distribution:", error);
            if (error.logs) {
                console.error("Transaction logs:", error.logs);
            }
            throw error;
        }
    });

    it("Claims rewards", async () => {
        try {
            const tracker = await program.account.contributionTracker.fetch(trackerPda);
            console.log("Current period number:", tracker.currentPeriod.toString());
    
            // Fetch contributor account to check last claimed period
            const [contributorPda] = PublicKey.findProgramAddressSync(
                [
                    Buffer.from("contributor"),
                    contributor.publicKey.toBuffer()
                ],
                program.programId
            );

            const contributorAccount = await program.account.contributor.fetch(contributorPda);
            console.log("Contributor last claimed period:", contributorAccount.lastClaimedPeriod.toString());
            console.log("Contributor current points:", contributorAccount.currentPeriodPoints.toString());
    
            // Calculate the correct period PDA
            const [periodPda] = PublicKey.findProgramAddressSync(
                [
                    Buffer.from("distribution_period"),
                    new anchor.BN(tracker.currentPeriod).toArrayLike(Buffer, 'le', 8)
                ],
                program.programId
            );
            console.log("Period PDA:", periodPda.toBase58());
    
            // Get and log period info
            const period = await program.account.distributionPeriod.fetch(periodPda);
            console.log("Period total points:", period.totalPoints.toString());
            console.log("Period tokens allocated:", period.tokensAllocated.toString());
            console.log("Period tokens distributed:", period.tokensDistributed.toString());
    
            // Get or create contributor token account
            const ata = await getAssociatedTokenAddress(
                mint,
                contributor.publicKey
            );
            
            let contributorTokenAccount;
            try {
                contributorTokenAccount = await getAccount(
                    provider.connection,
                    ata
                );
            } catch (e) {
                // Create if doesn't exist
                const account = await getOrCreateAssociatedTokenAccount(
                    provider.connection,
                    contributor,
                    mint,
                    contributor.publicKey
                );
                contributorTokenAccount = await getAccount(provider.connection, ata);
            }
            console.log("Contributor token account:", contributorTokenAccount.address.toBase58());
    
            const tx = await program.methods
                .claimRewards()
                .accounts({
                    contributor: contributor.publicKey,
                    contributionTracker: trackerPda,
                    periodToClaim: periodPda,
                    contributorAccount: contributorPda,
                    rewardVault: rewardVault,
                    contributorTokenAccount: ata,
                    rewardMint: mint,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    systemProgram: SystemProgram.programId,
                    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                })
                .signers([contributor])
                .rpc();
    
            await provider.connection.confirmTransaction(tx);
            console.log("Claim transaction:", tx);
    
            // Verify reward distribution
            const updatedPeriod = await program.account.distributionPeriod.fetch(periodPda);
            const updatedContributor = await program.account.contributor.fetch(contributorPda);
            const tokenBalance = await provider.connection.getTokenAccountBalance(ata);
    
            console.log("Updated period tokens distributed:", updatedPeriod.tokensDistributed.toString());
            console.log("Updated contributor last claimed period:", updatedContributor.lastClaimedPeriod.toString());
            console.log("Contributor token balance:", tokenBalance.value.uiAmount);
    
            expect(updatedPeriod.tokensDistributed.toNumber()).to.be.greaterThan(0);
            expect(tokenBalance.value.uiAmount).to.be.greaterThan(0);
    
        } catch (error) {
            console.error("Error in claim rewards:", error);
            if (error.logs) {
                console.error("Transaction logs:", error.logs);
            }
            throw error;
        }
    });
});
