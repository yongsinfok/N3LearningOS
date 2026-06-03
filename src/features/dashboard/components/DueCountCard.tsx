import { Card, CardContent } from "@/components/ui/card";

interface DueCountCardProps {
  dueCount: number;
  newAvailable: number;
}

export default function DueCountCard({ dueCount, newAvailable }: DueCountCardProps) {
  return (
    <Card className="bg-primary text-primary-foreground">
      <CardContent className="flex flex-col items-center gap-2 p-6">
        <p className="text-sm">今日待复习</p>
        <p className="text-5xl font-bold">{dueCount}</p>
        {newAvailable > 0 && (
          <p className="text-sm opacity-80">{newAvailable} 张新卡片</p>
        )}
      </CardContent>
    </Card>
  );
}
