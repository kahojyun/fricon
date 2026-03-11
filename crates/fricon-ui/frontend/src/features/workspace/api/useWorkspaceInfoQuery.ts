import { useQuery } from "@tanstack/react-query";
import { getWorkspaceInfo } from "./client";
import { workspaceKeys } from "./queryKeys";

export function useWorkspaceInfoQuery() {
  return useQuery({
    queryKey: workspaceKeys.info(),
    queryFn: getWorkspaceInfo,
    staleTime: Infinity,
    refetchOnWindowFocus: false,
  });
}
