import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { useKanjiList } from "../hooks/useKanji";
import KanjiGrid from "../components/KanjiGrid";
import ImportButton from "@/features/import/components/ImportButton";

export default function KanjiListPage() {
  const [search, setSearch] = useState("");
  const [page, setPage] = useState(1);
  const { data, isLoading, refetch } = useKanjiList(page, search || undefined);
  const navigate = useNavigate();

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">汉字</h1>
        <ImportButton contentType="kanji" onImportComplete={() => refetch()} />
      </div>

      <Input
        placeholder="搜索汉字..."
        value={search}
        onChange={(e) => {
          setSearch(e.target.value);
          setPage(1);
        }}
      />

      {isLoading ? (
        <div className="grid grid-cols-4 gap-3 sm:grid-cols-6 md:grid-cols-8 lg:grid-cols-10">
          {Array.from({ length: 20 }).map((_, i) => (
            <Skeleton key={i} className="h-20 w-full" />
          ))}
        </div>
      ) : data && data.items.length > 0 ? (
        <>
          <KanjiGrid kanjiList={data.items} onSelect={(id) => navigate(`/kanji/${id}`)} />
          {data.total > data.page_size && (
            <div className="flex justify-center gap-2">
              <Button
                variant="outline"
                disabled={page <= 1}
                onClick={() => setPage((p) => p - 1)}
              >
                上一页
              </Button>
              <span className="flex items-center text-sm">
                {page} / {Math.ceil(data.total / data.page_size)}
              </span>
              <Button
                variant="outline"
                disabled={page * data.page_size >= data.total}
                onClick={() => setPage((p) => p + 1)}
              >
                下一页
              </Button>
            </div>
          )}
        </>
      ) : (
        <div className="py-12 text-center text-muted-foreground">
          {search ? "没有找到匹配的汉字" : "暂无汉字数据"}
        </div>
      )}
    </div>
  );
}
