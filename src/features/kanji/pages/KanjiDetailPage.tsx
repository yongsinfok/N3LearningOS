import { useParams, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Separator } from "@/components/ui/separator";
import { Badge } from "@/components/ui/badge";
import { useKanjiDetail } from "../hooks/useKanji";
import WritingCanvas from "../components/WritingCanvas";

export default function KanjiDetailPage() {
  const { id } = useParams<{ id: string }>();
  const { data, isLoading, error } = useKanjiDetail(id!);
  const navigate = useNavigate();

  if (isLoading) {
    return (
      <div className="space-y-4">
        <Skeleton className="h-8 w-32" />
        <Skeleton className="h-96 w-full max-w-lg" />
      </div>
    );
  }

  if (error || !data) {
    return (
      <div className="flex flex-col items-center gap-3 py-12">
        <p className="text-destructive">汉字未找到</p>
        <Button variant="outline" onClick={() => navigate("/kanji")}>
          返回列表
        </Button>
      </div>
    );
  }

  let onyomiList: string[] = [];
  let kunyomiList: string[] = [];
  let exampleWordList: string[] = [];
  try {
    if (data.onyomi) onyomiList = JSON.parse(data.onyomi);
  } catch {}
  try {
    if (data.kunyomi) kunyomiList = JSON.parse(data.kunyomi);
  } catch {}
  try {
    if (data.example_words) exampleWordList = JSON.parse(data.example_words);
  } catch {}

  return (
    <div className="space-y-6">
      <Button variant="ghost" onClick={() => navigate("/kanji")}>
        ← 返回
      </Button>

      <Card className="mx-auto max-w-2xl">
        <CardContent className="space-y-6 p-8">
          <div className="flex items-center justify-center gap-8">
            <p className="text-7xl font-bold">{data.character}</p>
            <div className="space-y-2">
              {onyomiList.length > 0 && (
                <div className="flex flex-wrap gap-1">
                  <span className="text-sm text-muted-foreground">音读:</span>
                  {onyomiList.map((o) => (
                    <Badge key={o} variant="secondary">
                      {o}
                    </Badge>
                  ))}
                </div>
              )}
              {kunyomiList.length > 0 && (
                <div className="flex flex-wrap gap-1">
                  <span className="text-sm text-muted-foreground">训读:</span>
                  {kunyomiList.map((k) => (
                    <Badge key={k} variant="outline">
                      {k}
                    </Badge>
                  ))}
                </div>
              )}
              <p className="text-lg text-muted-foreground">{data.meaning}</p>
            </div>
          </div>

          <Separator />

          <div>
            <h3 className="mb-3 font-semibold">书写练习</h3>
            <div className="flex justify-center">
              <WritingCanvas character={data.character} />
            </div>
          </div>

          {exampleWordList.length > 0 && (
            <>
              <Separator />
              <div>
                <h3 className="mb-2 font-semibold">例词</h3>
                <div className="flex flex-wrap gap-2">
                  {exampleWordList.map((w) => (
                    <span key={w} className="rounded-md bg-muted px-3 py-1 text-sm">
                      {w}
                    </span>
                  ))}
                </div>
              </div>
            </>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
