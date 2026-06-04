import { useState } from "react";
import { Input } from "@/components/ui/input";
import { Skeleton } from "@/components/ui/skeleton";
import { useGrammarList } from "../hooks/useGrammar";
import GrammarRow from "../components/GrammarRow";
import ImportButton from "@/features/import/components/ImportButton";
export default function GrammarListPage() {
  const [search, setSearch] = useState("");
  const [page, setPage] = useState(1);
  const { data, isLoading, refetch } = useGrammarList(page, search || undefined);
  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold">文法</h1>
        <ImportButton contentType="grammar" onImportComplete={() => refetch()} />
      </div>
      <Input placeholder="搜索文法..." value={search} onChange={(e) => { setSearch(e.target.value); setPage(1); }} />
      {isLoading ? (
        <div className="space-y-2">{Array.from({ length: 5 }).map((_, i) => <Skeleton key={i} className="h-16 w-full" />)}</div>
      ) : data && data.items.length > 0 ? (
        <div className="space-y-2">{data.items.map((g) => <GrammarRow key={g.id} grammar={g} />)}</div>
      ) : (
        <div className="py-12 text-center text-muted-foreground">{search ? "没有找到匹配的文法" : "暂无文法数据"}</div>
      )}
    </div>
  );
}
