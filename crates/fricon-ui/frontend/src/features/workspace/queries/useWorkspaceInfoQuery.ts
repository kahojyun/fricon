import { useQuery } from "@tanstack/react-query";
import { getWorkspaceInfo } from "@/lib/backend";

export function useWorkspaceInfoQuery() {
  return useQuery({
    queryKey: ["workspaceInfo"],
    queryFn: getWorkspaceInfo,
    staleTime: Infinity,
    refetchOnWindowFocus: false,
  });
}
