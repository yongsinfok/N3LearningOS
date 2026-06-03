import { useQuery } from "@tanstack/react-query";
import { listGrammar, getGrammarDetail } from "@/services/api";

export function useGrammarList(page = 1, search?: string) {
  return useQuery({
    queryKey: ["grammar", "list", page, search],
    queryFn: () => listGrammar(page, search, 20),
  });
}

export function useGrammarDetail(id: string) {
  return useQuery({
    queryKey: ["grammar", id],
    queryFn: () => getGrammarDetail(id),
    enabled: !!id,
  });
}
