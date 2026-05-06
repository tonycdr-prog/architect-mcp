export type V10ProductArea =
  | "product-api"
  | "storage"
  | "accounts"
  | "remote-policy"
  | "github"
  | "billing"
  | "dashboard"
  | "cli"
  | "sdk"
  | "mcp-loop";

export type V10RouteMethod = "GET" | "POST" | "PATCH" | "DELETE";

export type V10RolloutMode = "suggest" | "warn" | "block";

export type V10ReadinessStatus = "pass" | "warn" | "fail";

export type V10Finding = {
  code: string;
  status: V10ReadinessStatus;
  area: V10ProductArea;
  message: string;
  recommendation: string;
};

export type V10ApiRoute = {
  method: V10RouteMethod;
  path: string;
  area: V10ProductArea;
  auth: "public" | "user" | "org-role" | "webhook-signature";
  tenantScoped: boolean;
  repositoryBoundary: string;
  idempotency?: boolean;
  notes: string[];
};

export type V10StorageEntity = {
  table: string;
  area: V10ProductArea;
  orgScoped: boolean;
  purpose: string;
  sensitiveColumns: string[];
  retention: string;
};

export type V10RepositoryBoundary = {
  name: string;
  owns: string[];
  forbiddenConsumers: string[];
  requiredChecks: string[];
};

export type V10DashboardScreen = {
  route: string;
  title: string;
  primerComponents: string[];
  dataSources: string[];
  accessibilityChecks: string[];
};

export type V10ImplementationSlice = {
  id: string;
  title: string;
  areas: V10ProductArea[];
  likelyFiles: string[];
  mcpToolsBefore: string[];
  mcpToolsAfter: string[];
  verification: string[];
  stopCondition: string;
};
