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

interface AdminOrganizationsProps {
	organizations: ReadonlyArray<Schemas.AdminOrganizationSummaryResponse>
	pagination: Schemas.PaginationMetadata | null | undefined
	search: string
	loading: boolean
	error: string | null
	onSearch: (value: string) => void
	onPage: (page: number) => void
	onSelect: (organization: Schemas.AdminOrganizationSummaryResponse) => void
}

export function AdminOrganizations({
	organizations,
	pagination,
	search,
	loading,
	error,
	onSearch,
	onPage,
	onSelect,
}: AdminOrganizationsProps) {
	return (
		<div className="mx-auto flex w-full max-w-5xl flex-col gap-4 p-6">
			<div>
				<h1 className="font-semibold text-xl">Organisations</h1>
				<p className="text-muted-foreground text-sm">
					Toutes les organisations de la plateforme et leurs flags.
				</p>
			</div>
			<AdminSearch
				value={search}
				placeholder="Rechercher par nom ou identifiant"
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
								<TableHead>Identifiant</TableHead>
								<TableHead className="text-right">Membres</TableHead>
								<TableHead>Créée le</TableHead>
								<TableHead className="pr-4">Flags</TableHead>
							</TableRow>
						</TableHeader>
						<TableBody>
							{organizations.length === 0 ? (
								<TableRow className="hover:bg-transparent">
									<TableCell
										colSpan={5}
										className="py-8 text-center text-muted-foreground"
									>
										Aucune organisation.
									</TableCell>
								</TableRow>
							) : (
								organizations.map((organization) => (
									<TableRow
										key={organization.id}
										className="cursor-pointer"
										onClick={() => onSelect(organization)}
									>
										<TableCell className="pl-4 font-medium">
											<button
												type="button"
												className="text-left outline-none focus-visible:underline"
												onClick={(event) => {
													event.stopPropagation()
													onSelect(organization)
												}}
											>
												{organization.name}
											</button>
										</TableCell>
										<TableCell className="text-muted-foreground">
											{organization.slug}
										</TableCell>
										<TableCell className="tabular text-right">
											{organization.member_count}
										</TableCell>
										<TableCell>
											{formatAdminDate(organization.created_at)}
										</TableCell>
										<TableCell className="pr-4">
											<div className="flex flex-wrap gap-1">
												{organization.features.map((feature) => (
													<Badge key={feature} variant="secondary">
														{feature}
													</Badge>
												))}
											</div>
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
