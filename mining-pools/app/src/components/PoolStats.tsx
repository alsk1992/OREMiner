'use client';

import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';

interface PoolData {
  type: '25' | '18';
  name: string;
  totalRounds: number;
  totalWins: number;
  totalOre: number;
}

interface PoolStatsProps {
  pool25: PoolData;
  pool18: PoolData;
}

export default function PoolStats({ pool25, pool18 }: PoolStatsProps) {
  // Mock historical data - replace with actual data
  const chartData = [
    { time: '00:00', pool25: 0, pool18: 0 },
    { time: '04:00', pool25: 0.05, pool18: 0.04 },
    { time: '08:00', pool25: 0.12, pool18: 0.09 },
    { time: '12:00', pool25: 0.21, pool18: 0.17 },
    { time: '16:00', pool25: 0.32, pool18: 0.25 },
    { time: '20:00', pool25: 0.42, pool18: 0.31 },
    { time: '24:00', pool25: 0.52, pool18: 0.38 },
  ];

  return (
    <div className="bg-gray-800 rounded-lg p-6 border border-gray-700">
      <h2 className="text-2xl font-bold text-white mb-6">
        📈 Live Performance (Last 24h)
      </h2>

      {/* Chart */}
      <div className="mb-8">
        <ResponsiveContainer width="100%" height={300}>
          <LineChart data={chartData}>
            <CartesianGrid strokeDasharray="3 3" stroke="#374151" />
            <XAxis dataKey="time" stroke="#9CA3AF" />
            <YAxis stroke="#9CA3AF" />
            <Tooltip
              contentStyle={{ backgroundColor: '#1F2937', border: '1px solid #374151' }}
              labelStyle={{ color: '#F3F4F6' }}
            />
            <Legend />
            <Line
              type="monotone"
              dataKey="pool25"
              stroke="#10B981"
              strokeWidth={2}
              name="25-Square Pool"
              dot={{ fill: '#10B981' }}
            />
            <Line
              type="monotone"
              dataKey="pool18"
              stroke="#8B5CF6"
              strokeWidth={2}
              name="18-Square Pool"
              dot={{ fill: '#8B5CF6' }}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>

      {/* Comparison Table */}
      <div className="grid md:grid-cols-2 gap-6">
        <div className="bg-gray-900 rounded-lg p-4">
          <h3 className="text-lg font-semibold text-green-400 mb-4">
            🏦 25-Square Pool
          </h3>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-400">Total Rounds:</span>
              <span className="text-white font-semibold">{pool25.totalRounds}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Total Wins:</span>
              <span className="text-green-400 font-semibold">{pool25.totalWins}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Win Rate:</span>
              <span className="text-white font-semibold">
                {pool25.totalRounds > 0
                  ? ((pool25.totalWins / pool25.totalRounds) * 100).toFixed(1)
                  : 0}%
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">ORE Earned:</span>
              <span className="text-yellow-400 font-semibold">{pool25.totalOre.toFixed(4)}</span>
            </div>
          </div>
        </div>

        <div className="bg-gray-900 rounded-lg p-4">
          <h3 className="text-lg font-semibold text-purple-400 mb-4">
            ⚡ 18-Square Pool
          </h3>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-gray-400">Total Rounds:</span>
              <span className="text-white font-semibold">{pool18.totalRounds}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Total Wins:</span>
              <span className="text-green-400 font-semibold">{pool18.totalWins}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">Win Rate:</span>
              <span className="text-white font-semibold">
                {pool18.totalRounds > 0
                  ? ((pool18.totalWins / pool18.totalRounds) * 100).toFixed(1)
                  : 0}%
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-gray-400">ORE Earned:</span>
              <span className="text-yellow-400 font-semibold">{pool18.totalOre.toFixed(4)}</span>
            </div>
          </div>
        </div>
      </div>

      {/* Key Metrics */}
      <div className="mt-6 grid grid-cols-3 gap-4">
        <div className="bg-gradient-to-br from-green-900 to-green-800 rounded-lg p-4 text-center">
          <p className="text-green-200 text-sm mb-1">Total Pool TVL</p>
          <p className="text-2xl font-bold text-white">74.1 SOL</p>
          <p className="text-green-300 text-xs">≈ $11,720</p>
        </div>
        <div className="bg-gradient-to-br from-yellow-900 to-yellow-800 rounded-lg p-4 text-center">
          <p className="text-yellow-200 text-sm mb-1">Total ORE Earned</p>
          <p className="text-2xl font-bold text-white">0.737 ORE</p>
          <p className="text-yellow-300 text-xs">≈ $341</p>
        </div>
        <div className="bg-gradient-to-br from-purple-900 to-purple-800 rounded-lg p-4 text-center">
          <p className="text-purple-200 text-sm mb-1">Active Miners</p>
          <p className="text-2xl font-bold text-white">23</p>
          <p className="text-purple-300 text-xs">Depositors</p>
        </div>
      </div>
    </div>
  );
}
