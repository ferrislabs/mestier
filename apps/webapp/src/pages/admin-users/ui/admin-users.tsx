import type { Schemas } from '#/api/api.client'
import { Badge } from '#/components/ui/badge'
import { Skeleton } from '#/components/ui/skeleton'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import { formatAdminDate } from '#/pages/admin-layout/lib/admin-format'
import {
	AdminPagination,
	AdminSearch,
} from '#/pages/admin-layout/ui/admin-list-toolbar'

interface AdminUsersProps {
	users: ReadonlyArray<Schemas.AdminUserResponse>
	pagination: Schemas.PaginationMetadata | null | undefined
	search: string
	loading: boolean
	error: string | null
	onSearch: (value: string) => void
	onPage: (page: number) => void
}

export function AdminUsers({
	users,
	pagination,
	search,
	loading,
	error,
	onSearch,
	onPage,
}: AdminUsersProps) {
	return (
		<div className="mx-auto flex w-full max-w-5xl flex-col gap-4 p-6">
			<div>
				<h1 className="font-semibold text-xl">Utilisateurs</h1>
				<p className="text-muted-foreground text-sm">
					Tous les comptes de la plateforme et leurs organisations.
				</p>
			</div>
			<AdminSearch
				value={search}
				placeholder="Rechercher par nom, email ou identifiant"
				onChange={onSearch}
			/>
			{error ? (
				<p role="alert" className="text-sm text-state-alert">
					{error}
				</p>
			) : null}
			{loading ? (
				<Skeleton className="h-48 w-full" />
			) : (
				<div className="overflow-hidden rounded-lg border bg-card">
					<Table>
						<TableHeader>
							<TableRow className="hover:bg-transparent">
								<TableHead className="pl-4">Nom</TableHead>
								<TableHead>Email</TableHead>
								<TableHead>Identifiant</TableHead>
								<TableHead>Organisations</TableHead>
								<TableHead className="pr-4">Inscrit le</TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{users.length === 0 ? (
								<TableRow className="hover:bg-transparent">
									<TableCell
										colSpan={5}
										className="py-8 text-center text-muted-foreground"
									>
										Aucun utilisateur.
									</TableCell>
								</TableRow>
							) : (
								users.map((user) => (
									<TableRow key={user.id}>
										<TableCell className="pl-4 font-medium">
											{user.display_name}
										</TableCell>
										<TableCell className="text-muted-foreground">
											{user.email}
										</TableCell>
										<TableCell className="text-muted-foreground">
											{user.username}
										</TableCell>
										<TableCell>
											<div className="flex flex-wrap gap-1">
												{user.organizations.map((organization) => (
													<Badge key={organization.slug} variant="secondary">
														{organization.name}
													</Badge>
												))}
											</div>
										</TableCell>
										<TableCell className="pr-4">
											{formatAdminDate(user.created_at)}
										</TableCell>
									</TableRow>
								))
							)}
						</TableBody>
					</Table>
				</div>
			)}
			<AdminPagination pagination={pagination} onPage={onPage} />
		</div>
	)
}
