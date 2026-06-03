import { useQuery } from "@tanstack/react-query";
import { globalSearch, SearchResult } from "@/services/api";

export function useGlobalSearch(query: string) {
  return useQuery<SearchResult[]>({
    queryKey: ["search", query],
    queryFn: () => globalSearch(query),
    enabled: query.length >= 2,
  });
}
