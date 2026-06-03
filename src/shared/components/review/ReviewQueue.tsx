import { Progress } from "@/components/ui/progress";

interface ReviewQueueProps {
  current: number;
  total: number;
}

export default function ReviewQueue({ current, total }: ReviewQueueProps) {
  const pct = total > 0 ? Math.round((current / total) * 100) : 0;
  return (
    <div className="mb-4">
      <div className="mb-1 flex items-center justify-between text-sm text-muted-foreground">
        <span>{current} / {total}</span>
        <span>{pct}%</span>
      </div>
      <Progress value={pct} className="h-2" />
    </div>
  );
}
