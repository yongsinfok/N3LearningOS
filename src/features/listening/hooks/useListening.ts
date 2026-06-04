import { useQuery } from "@tanstack/react-query";
import { listListening, getListeningDetail } from "@/services/api";

export function useListeningList(page = 1, search?: string) {
  return useQuery({
    queryKey: ["listening", "list", page, search],
    queryFn: () => listListening(page, search, 20),
  });
}

export function useListeningDetail(id: string) {
  return useQuery({
    queryKey: ["listening", id],
    queryFn: () => getListeningDetail(id),
    enabled: !!id,
  });
}
