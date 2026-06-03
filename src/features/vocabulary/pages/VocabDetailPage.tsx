import { useParams, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { useVocabDetail } from "../hooks/useVocabulary";

export default function VocabDetailPage() {
  const { id } = useParams<{ id: string }>();
  const { data, isLoading, error } = useVocabDetail(id!);
  const navigate = useNavigate();

  if (isLoading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-8 w-32" />
        <Skeleton className="h-64 w-full max-w-lg" />
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="flex flex-col items-center gap-3 py-12">
        <p className="text-destructive">词汇未找到</p>
        <Button variant="outline" onClick={() => navigate("/vocabulary")}>返回列表</Button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <Button variant="ghost" onClick={() => navigate("/vocabulary")}>← 返回</Button>
      <Card className="mx-auto max-w-lg">
        <CardContent className="flex flex-col items-center gap-4 p-8">
          <p className="text-5xl font-bold">{data.word}</p>
          <div className="space-y-2 text-center">
            <p className="text-xl text-muted-foreground">{data.reading}</p>
            <p className="text-lg">{data.meaning}</p>
          </div>
          {data.example && (
            <div className="mt-4 rounded-lg bg-muted p-4">
              <p className="text-sm">{data.example}</p>
            </div>
          )}
        </CardContent>
      </Card>
      <div className="flex justify-center gap-3">
        <Button onClick={() => navigate("/vocabulary/quiz")}>开始测验</Button>
      </div>
    </div>
  );
}
