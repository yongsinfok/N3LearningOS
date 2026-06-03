import { useState } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { useNavigate } from "react-router-dom";
import { useVocabList } from "../hooks/useVocabulary";
import VocabRow from "../components/VocabRow";

export default function VocabListPage() {
  const [search, setSearch] = useState("");
  const [page, setPage] = useState(1);
  const { data, isLoading } = useVocabList(page, search || undefined);
  const navigate = useNavigate();

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">词汇</h1>
        <Button onClick={() => navigate("/vocabulary/quiz")}>开始测验</Button>
      </div>
      <Input
        placeholder="搜索词汇..."
        value={search}
        onChange={(e) => { setSearch(e.target.value); setPage(1); }}
      />
      {isLoading ? (
        <div className="space-y-2">
          {Array.from({ length: 5 }).map((_, i) => <Skeleton key={i} className="h-16 w-full" />)}
        </div>
      ) : data && data.items.length > 0 ? (
        <>
          <div className="space-y-2">
            {data.items.map((v) => <VocabRow key={v.id} vocab={v} />)}
          </div>
          {data.total > data.page_size && (
            <div className="flex justify-center gap-2">
              <Button variant="outline" disabled={page <= 1} onClick={() => setPage(p => p - 1)}>上一页</Button>
              <span className="flex items-center text-sm">{page} / {Math.ceil(data.total / data.page_size)}</span>
              <Button variant="outline" disabled={page * data.page_size >= data.total} onClick={() => setPage(p => p + 1)}>下一页</Button>
            </div>
          )}
        </>
      ) : (
        <div className="py-12 text-center text-muted-foreground">
          {search ? "没有找到匹配的词汇" : "暂无词汇数据，请确认内容已正确导入"}
        </div>
      )}
    </div>
  );
}
