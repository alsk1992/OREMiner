'use client';

import { useState } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import PoolCard from './PoolCard';
import PoolStats from './PoolStats';

export default function PoolDashboard() {
  const { connected } = useWallet();
  const [selectedPool, setSelectedPool] = useState<'25' | '18' | null>(null);

  // Mock data - replace with actual on-chain data
  const pool25Data = {
    type: '25' as const,
    name: 'The Banker',
    description: '100% win rate • Cover all squares',
    winRate: 100,
    avgReturn: 2.5,
    risk: 1,
    poolSize: 45.8,
    yourShare: 0,
    totalRounds: 147,
    totalWins: 147,
    totalOre: 0.425,
  };

  const pool18Data = {
    type: '18' as const,
    name: 'The Grinder',
    description: '72% win rate • 18 least crowded',
    winRate: 72,
    avgReturn: 8.5,
    risk: 3,
    poolSize: 28.3,
    yourShare: 0,
    totalRounds: 89,
    totalWins: 64,
    totalOre: 0.312,
  };

  return (
    <div className="space-y-8">
      {/* Wallet Connection */}
      <div className="flex justify-center">
        <WalletMultiButton className="!bg-purple-600 hover:!bg-purple-700" />
      </div>

      {/* Pool Selection */}
      <div className="grid md:grid-cols-2 gap-6">
        <PoolCard
          pool={pool25Data}
          isSelected={selectedPool === '25'}
          onSelect={() => setSelectedPool('25')}
          connected={connected}
        />
        <PoolCard
          pool={pool18Data}
          isSelected={selectedPool === '18'}
          onSelect={() => setSelectedPool('18')}
          connected={connected}
        />
      </div>

      {/* Stats Dashboard */}
      {connected && (
        <div className="bg-gray-800 rounded-lg p-6 border border-purple-500">
          <h2 className="text-2xl font-bold text-white mb-6">
            📊 Your Portfolio
          </h2>
          <div className="grid md:grid-cols-3 gap-6">
            <div className="bg-gray-900 rounded-lg p-4">
              <p className="text-gray-400 text-sm">Total Deposited</p>
              <p className="text-3xl font-bold text-white">0.00 SOL</p>
            </div>
            <div className="bg-gray-900 rounded-lg p-4">
              <p className="text-gray-400 text-sm">Total Earnings</p>
              <p className="text-3xl font-bold text-green-400">+0.00 SOL</p>
            </div>
            <div className="bg-gray-900 rounded-lg p-4">
              <p className="text-gray-400 text-sm">ORE Earned</p>
              <p className="text-3xl font-bold text-yellow-400">0.0000 ORE</p>
            </div>
          </div>
        </div>
      )}

      {/* Live Stats */}
      <PoolStats pool25={pool25Data} pool18={pool18Data} />

      {/* How It Works */}
      <div className="bg-gray-800 rounded-lg p-8 border border-gray-700">
        <h2 className="text-3xl font-bold text-white mb-6">
          How It Works
        </h2>
        <div className="grid md:grid-cols-4 gap-6">
          <div className="text-center">
            <div className="text-4xl mb-3">💰</div>
            <h3 className="text-lg font-semibold text-white mb-2">1. Deposit</h3>
            <p className="text-gray-400 text-sm">
              Add SOL to Pool A or Pool B (or both)
            </p>
          </div>
          <div className="text-center">
            <div className="text-4xl mb-3">⛏️</div>
            <h3 className="text-lg font-semibold text-white mb-2">2. Mine</h3>
            <p className="text-gray-400 text-sm">
              Pool automatically mines using optimal strategy
            </p>
          </div>
          <div className="text-center">
            <div className="text-4xl mb-3">📈</div>
            <h3 className="text-lg font-semibold text-white mb-2">3. Earn</h3>
            <p className="text-gray-400 text-sm">
              Rewards auto-distribute to your share
            </p>
          </div>
          <div className="text-center">
            <div className="text-4xl mb-3">💸</div>
            <h3 className="text-lg font-semibold text-white mb-2">4. Withdraw</h3>
            <p className="text-gray-400 text-sm">
              Claim your share + earnings anytime
            </p>
          </div>
        </div>
      </div>

      {/* Trust Indicators */}
      <div className="bg-gradient-to-r from-purple-900 to-blue-900 rounded-lg p-6 border border-purple-500">
        <div className="grid md:grid-cols-3 gap-6 text-center">
          <div>
            <div className="text-3xl mb-2">🔒</div>
            <h3 className="text-lg font-semibold text-white">Trustless</h3>
            <p className="text-sm text-gray-300">Smart contract enforced</p>
          </div>
          <div>
            <div className="text-3xl mb-2">🔍</div>
            <h3 className="text-lg font-semibold text-white">Transparent</h3>
            <p className="text-sm text-gray-300">All transactions on-chain</p>
          </div>
          <div>
            <div className="text-3xl mb-2">⚡</div>
            <h3 className="text-lg font-semibold text-white">Fair</h3>
            <p className="text-sm text-gray-300">Proportional rewards</p>
          </div>
        </div>
      </div>
    </div>
  );
}
