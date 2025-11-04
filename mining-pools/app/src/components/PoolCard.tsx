'use client';

import { useState } from 'react';

interface PoolData {
  type: '25' | '18';
  name: string;
  description: string;
  winRate: number;
  avgReturn: number;
  risk: number;
  poolSize: number;
  yourShare: number;
  totalRounds: number;
  totalWins: number;
  totalOre: number;
}

interface PoolCardProps {
  pool: PoolData;
  isSelected: boolean;
  onSelect: () => void;
  connected: boolean;
}

export default function PoolCard({ pool, isSelected, onSelect, connected }: PoolCardProps) {
  const [depositAmount, setDepositAmount] = useState('');
  const [showDeposit, setShowDeposit] = useState(false);

  const handleDeposit = () => {
    if (!connected) {
      alert('Please connect your wallet first');
      return;
    }
    // TODO: Call smart contract deposit function
    console.log(`Depositing ${depositAmount} SOL to ${pool.name}`);
    setShowDeposit(false);
    setDepositAmount('');
  };

  const riskStars = '⭐'.repeat(pool.risk) + '☆'.repeat(5 - pool.risk);

  return (
    <div
      className={`bg-gray-800 rounded-lg p-6 border-2 transition-all cursor-pointer hover:shadow-xl ${
        isSelected ? 'border-purple-500 shadow-purple-500/50' : 'border-gray-700'
      }`}
      onClick={onSelect}
    >
      {/* Header */}
      <div className="flex justify-between items-start mb-4">
        <div>
          <h2 className="text-2xl font-bold text-white">{pool.name}</h2>
          <p className="text-gray-400 text-sm">{pool.description}</p>
        </div>
        <div className="text-3xl">{pool.type === '25' ? '🏦' : '⚡'}</div>
      </div>

      {/* Stats */}
      <div className="space-y-3 mb-6">
        <div className="flex justify-between items-center">
          <span className="text-gray-400">Win Rate:</span>
          <span className="text-white font-semibold">{pool.winRate}%</span>
        </div>
        <div className="flex justify-between items-center">
          <span className="text-gray-400">Avg Return:</span>
          <span className="text-green-400 font-semibold">+{pool.avgReturn}%/round</span>
        </div>
        <div className="flex justify-between items-center">
          <span className="text-gray-400">Risk:</span>
          <span className="text-yellow-400">{riskStars}</span>
        </div>
        <div className="flex justify-between items-center">
          <span className="text-gray-400">Pool Size:</span>
          <span className="text-white font-semibold">{pool.poolSize} SOL</span>
        </div>
        <div className="flex justify-between items-center">
          <span className="text-gray-400">Your Share:</span>
          <span className="text-purple-400 font-semibold">{pool.yourShare}%</span>
        </div>
      </div>

      {/* Performance */}
      <div className="bg-gray-900 rounded-lg p-4 mb-6">
        <p className="text-gray-400 text-sm mb-2">Performance (Last 24h)</p>
        <div className="grid grid-cols-3 gap-2 text-center">
          <div>
            <p className="text-xs text-gray-500">Rounds</p>
            <p className="text-white font-semibold">{pool.totalRounds}</p>
          </div>
          <div>
            <p className="text-xs text-gray-500">Wins</p>
            <p className="text-green-400 font-semibold">{pool.totalWins}</p>
          </div>
          <div>
            <p className="text-xs text-gray-500">ORE</p>
            <p className="text-yellow-400 font-semibold">{pool.totalOre}</p>
          </div>
        </div>
      </div>

      {/* Deposit Interface */}
      {!showDeposit ? (
        <button
          onClick={(e) => {
            e.stopPropagation();
            setShowDeposit(true);
          }}
          className="w-full bg-purple-600 hover:bg-purple-700 text-white font-semibold py-3 px-4 rounded-lg transition-colors"
          disabled={!connected}
        >
          {connected ? 'Deposit SOL' : 'Connect Wallet to Deposit'}
        </button>
      ) : (
        <div onClick={(e) => e.stopPropagation()} className="space-y-3">
          <input
            type="number"
            value={depositAmount}
            onChange={(e) => setDepositAmount(e.target.value)}
            placeholder="Amount (SOL)"
            className="w-full bg-gray-700 text-white px-4 py-2 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500"
            step="0.01"
            min="0.01"
          />
          <div className="grid grid-cols-2 gap-2">
            <button
              onClick={handleDeposit}
              className="bg-green-600 hover:bg-green-700 text-white font-semibold py-2 px-4 rounded-lg transition-colors"
            >
              Confirm
            </button>
            <button
              onClick={() => {
                setShowDeposit(false);
                setDepositAmount('');
              }}
              className="bg-gray-600 hover:bg-gray-700 text-white font-semibold py-2 px-4 rounded-lg transition-colors"
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* Quick Info */}
      <div className="mt-4 pt-4 border-t border-gray-700">
        <p className="text-xs text-gray-500 text-center">
          {pool.type === '25'
            ? '🔒 Guaranteed wins • Best for steady ORE accumulation'
            : '⚡ Higher returns • Best for active traders'
          }
        </p>
      </div>
    </div>
  );
}
