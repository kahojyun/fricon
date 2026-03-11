import { commands, type WorkspaceInfo } from "@/shared/lib/bindings";
import { invoke } from "@/shared/lib/tauri";

export async function getWorkspaceInfo(): Promise<WorkspaceInfo> {
  return invoke(commands.getWorkspaceInfo());
}
