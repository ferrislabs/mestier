import { Link } from '@tanstack/react-router'
import { ArrowLeft } from 'lucide-react'
import type { Schemas } from '#/api/api.client'
import { Badge } from '#/components/ui/badge'
import { Skeleton } from '#/components/ui/skeleton'
import { Switch } from '#/components/ui/switch'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import {
	formatAdminDate,
	formatAdminDateTime,
} from '#/pages/admin-layout/lib/admin-format'
import {
	featureStateLine,
	historySentence,
} from '#/pages/admin-organizations/lib/history'

interface AdminOrganizationProps {
	organization: Schemas.AdminOrganizationResponse | undefined
	loading: boolean
	error: string | null
	pendingFlag: string | null
	onToggle: (flag: string, enabled: boolean) => void
}

export function AdminOrganization({
	organization,
	loading,
	error,
	pendingFlag,
	onToggle,
}: AdminOrganizationProps) {
	return (
		<div className="mx-auto flex w-full max-w-5xl flex-col gap-6 p-6">
			<Link
				to="/admin/organisations"
				className="inline-flex w-fit items-center gap-1.5 text-muted-foreground text-sm hover:text-foreground"
			>
				<ArrowLeft className="size-4" aria-hidden />
				Organisations
			</Link>
			{error ? (
				<p role="alert" className="text-sm text-state-alert">
					{error}
				</p>
			) : null}
			{loading || !organization ? (
				<Skeleton className="h-64 w-full" />
			) : (
				<>
					<div>
						<h1 className="font-semibold text-xl">{organization.name}</h1>
						<p className="text-muted-foreground text-sm">
							{organization.slug} · créée le{' '}
							{formatAdminDate(organization.created_at)}
						</p>
					</div>

					<section className="flex flex-col gap-2">
						<h2 className="font-medium text-sm">Flags</h2>
						<ul className="divide-y overflow-hidden rounded-lg border bg-card">
							{organization.features.map((feature) => {
								const line = featureStateLine(feature, formatAdminDate)
								return (
									<li
										key={feature.key}
										className="flex items-center justify-between gap-4 px-4 py-3"
									>
										<div className="min-w-0">
											<p className="font-medium text-sm">{feature.label}</p>
											<p className="text-muted-foreground text-xs">
												{feature.description}
											</p>
											{line ? (
												<p className="text-muted-foreground text-xs">{line}</p>
											) : null}
										</div>
										<Switch
											aria-label={feature.label}
											checked={feature.enabled}
											disabled={pendingFlag === feature.key}
											onCheckedChange={(enabled) =>
												onToggle(feature.key, enabled)
											}
										/>
									</li>
								)
							})}
						</ul>
					</section>

					<section className="flex flex-col gap-2">
						<h2 className="font-medium text-sm">
							Membres ({organization.members.length})
						</h2>
						<div className="overflow-hidden rounded-lg border bg-card">
							<Table>
								<TableHeader>
									<TableRow className="hover:bg-transparent">
										<TableHead className="pl-4">Nom</TableHead>
										<TableHead>Email</TableHead>
										<TableHead>Rôles</TableHead>
										<TableHead className="pr-4">Arrivé le</TableHead>
									</TableRow>
								</TableHeader>
								<TableBody>
									{organization.members.map((member) => (
										<TableRow key={`${member.name}-${member.created_at}`}>
											<TableCell className="pl-4 font-medium">
												{member.name}
											</TableCell>
											<TableCell className="text-muted-foreground">
												{member.email ?? '–'}
											</TableCell>
											<TableCell>
												<div className="flex flex-wrap gap-1">
													{member.roles.map((role) => (
														<Badge key={role} variant="secondary">
															{role}
														</Badge>
													))}
												</div>
											</TableCell>
											<TableCell className="pr-4">
												{formatAdminDate(member.created_at)}
											</TableCell>
										</TableRow>
									))}
								</TableBody>
							</Table>
						</div>
					</section>

					<section className="flex flex-col gap-2">
						<h2 className="font-medium text-sm">Historique des flags</h2>
						{organization.history.length === 0 ? (
							<p className="text-muted-foreground text-sm">
								Aucun changement pour l’instant.
							</p>
						) : (
							<ul className="divide-y overflow-hidden rounded-lg border bg-card">
								{organization.history.map((change) => (
									<li
										key={`${change.flag}-${change.occurred_at}`}
										className="flex items-center justify-between gap-4 px-4 py-2.5 text-sm"
									>
										<span>
											{historySentence(change, organization.features)}
										</span>
										<span className="shrink-0 text-muted-foreground text-xs">
											{formatAdminDateTime(change.occurred_at)}
										</span>
									</li>
								))}
							</ul>
						)}
					</section>
				</>
			)}
		</div>
	)
}
