import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

const MY_ORGS_KEY = "/api/v1/users/@me/organizations";

export function useMyOrganizations() {
  return useQuery(
    window.tanstackApi.get("/api/v1/users/@me/organizations").queryOptions,
  );
}

export function useCreateOrganization() {
  const queryClient = useQueryClient();

  return useMutation({
    ...window.tanstackApi.mutation("post", "/api/v1/organizations")
      .mutationOptions,
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: [{ _id: MY_ORGS_KEY }],
      });
    },
  });
}
