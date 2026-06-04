import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Input } from "@/components/ui/input";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { useListeningList } from "../hooks/useListening";

export default function ListeningListPage() {
  const [search, setSearch] = useState("");
  const [page, setPage] = useState(1);
  const { data, isLoading } = useListeningList(page, search || undefined);
  const navigate = useNavigate();

  return (
    <div className="space-y-4">
      <h1 className="text-2xl font-bold">听力</h1>
      <Input
        placeholder="搜索课程..."
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
          {data.items.map((item) => (
            <Card
              key={item.id}
              className="cursor-pointer transition-colors hover:bg-muted/50"
              onClick={() => navigate(`/listening/${item.id}`)}
            >
              <CardContent className="flex items-center justify-between p-4">
                <p className="font-medium">{item.title}</p>
                <Badge variant="secondary">{item.level}</Badge>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : (
        <div className="py-12 text-center text-muted-foreground">
          {search ? "没有找到匹配的课程" : "暂无听力课程"}
        </div>
      )}
    </div>
  );
}
