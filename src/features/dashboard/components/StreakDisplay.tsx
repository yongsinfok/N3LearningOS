interface StreakDisplayProps {
  streak: number;
}

export default function StreakDisplay({ streak }: StreakDisplayProps) {
  const fire = streak > 0 ? "🔥".repeat(Math.min(streak, 5)) : "💤";
  return (
    <div className="flex items-center gap-2 text-lg">
      <span className="text-3xl">{fire}</span>
      <span className="font-bold">{streak} 天连胜</span>
    </div>
  );
}
