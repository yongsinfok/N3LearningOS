import { useParams, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Separator } from "@/components/ui/separator";
import { useGrammarDetail } from "../hooks/useGrammar";
export default function GrammarDetailPage() {
  const { id } = useParams<{ id: string }>();
  const { data, isLoading, error } = useGrammarDetail(id!);
  const navigate = useNavigate();
  if (isLoading) {
    return <div className="space-y-4"><Skeleton className="h-8 w-32" /><Skeleton className="h-64 w-full max-w-lg" /></div>;
  }
  if (error || !data) {
    return <div className="flex flex-col items-center gap-3 py-12"><p className="text-destructive">文法未找到</p><Button variant="outline" onClick={() => navigate("/grammar")}>返回列表</Button></div>;
  }
  let examples: string[] = [];
  let relatedPatterns: string[] = [];
  try { examples = JSON.parse(data.examples); } catch {}
  try { if (data.related) relatedPatterns = JSON.parse(data.related); } catch {}
  return (
    <div className="space-y-6">
      <Button variant="ghost" onClick={() => navigate("/grammar")}>← 返回</Button>
      <Card className="mx-auto max-w-2xl">
        <CardContent className="space-y-6 p-8">
          <div><p className="font-mono text-3xl font-bold">{data.pattern}</p><p className="mt-1 text-lg text-muted-foreground">{data.meaning}</p></div>
          <Separator />
          <div><h3 className="mb-2 font-semibold">用法说明</h3><p className="text-sm leading-relaxed">{data.explanation}</p></div>
          {examples.length > 0 && (<><Separator /><div><h3 className="mb-2 font-semibold">例句</h3><ul className="space-y-2">{examples.map((ex, i) => (<li key={i} className="rounded-md bg-muted p-3 text-sm">{ex}</li>))}</ul></div></>)}
          {relatedPatterns.length > 0 && (<><Separator /><div><h3 className="mb-2 font-semibold">关联文法</h3><div className="flex flex-wrap gap-2">{relatedPatterns.map((rp) => (<span key={rp} className="rounded-md bg-secondary px-3 py-1 text-sm font-mono">{rp}</span>))}</div></div></>)}
        </CardContent>
      </Card>
    </div>
  );
}
