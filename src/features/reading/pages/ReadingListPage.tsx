import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Input } from "@/components/ui/input";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { useReadingList } from "../hooks/useReading";

export default function ReadingListPage() {
  const [search, setSearch] = useState("");
  const [page, setPage] = useState(1);
  const { data, isLoading } = useReadingList(page, search || undefined);
  const navigate = useNavigate();

  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-bold">阅读</h1>
      <Input
        placeholder="搜索文章..."
        value={search}
        onChange={(e) => { setSearch(e.target.value); setPage(1); }}
      />
      {isLoading ? (
        <div className="space-y-3">
          {Array.from({ length: 4 }).map((_, i) => (
            <Skeleton key={i} className="h-20 w-full" />
          ))}
        </div>
      ) : data && data.items.length > 0 ? (
        <div className="space-y-3">
          {data.items.map((article) => (
            <Card
              key={article.id}
              className="cursor-pointer transition-colors hover:bg-muted/50"
              onClick={() => navigate(`/reading/${article.id}`)}
            >
              <CardContent className="flex items-center justify-between p-4">
                <div>
                  <p className="font-medium">{article.title}</p>
                </div>
                <Badge variant="secondary">{article.level}</Badge>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : (
        <div className="py-12 text-center text-muted-foreground">
          {search ? "没有找到匹配的文章" : "暂无阅读文章"}
        </div>
      )}
    </div>
  );
}
