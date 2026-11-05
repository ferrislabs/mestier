import { Link } from '@tanstack/react-router'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import type { Customer } from '#/hooks/use-customers'
import { cn } from '#/lib/utils'
import { buildOrgPath } from '#/modules/org-path'
import type { AccountExtras } from '#/pages/customers/feature/mocks/account-extras'
import { formatDate } from '#/pages/customers/lib/format'
import {
	ACCOUNT_KIND_LABELS,
	accountStatusOf,
	customerPipelineStageLabel,
	kindOf,
} from '#/pages/customers/types'
import { hueOf, NameAvatar } from '#/pages/customers/ui/avatars'
import { Dash } from '#/pages/customers/ui/detail-card'
import { Money } from '#/pages/customers/ui/money'
import {
	AccountStatusBadge,
	StageBadge,
} from '#/pages/customers/ui/status-badges'

export interface Outstanding {
	cents: number
	oldestDueAt: string | null
}

export function CustomersTable({
	organizationSlug,
	customers,
	extrasOf,
	outstandingOf,
	now,
}: {
	organizationSlug: string
	customers: readonly Customer[]
	extrasOf: (customerId: string) => AccountExtras
	outstandingOf: (customerId: string) => Outstanding | undefined
	now: number
}) {
	return (
		<div className="overflow-hidden rounded-lg border bg-card">
			<Table>
				<TableHeader>
					<TableRow className="hover:bg-transparent">
						<TableHead className="pl-4">Client</TableHead>
						<TableHead className="max-xl:hidden">Contact</TableHead>
						<TableHead className="text-right">Encours</TableHead>
						<TableHead className="max-lg:hidden">Étape</TableHead>
						<TableHead className="max-lg:hidden">Créé le</TableHead>
						<TableHead className="pr-4">Statut</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{customers.map((customer) => {
						const outstanding = outstandingOf(customer.id)
						const late =
							outstanding?.oldestDueAt != null &&
							new Date(outstanding.oldestDueAt).getTime() < now
						const archived = customer.status === 'ARCHIVED'
						const kind = kindOf(customer, extrasOf(customer.id))
						return (
							<TableRow
								key={customer.id}
								className={cn(
									'relative cursor-pointer focus-within:bg-muted/50',
									archived && 'opacity-60',
								)}
							>
								<TableCell className="py-2.5 pl-4">
									<div className="flex items-center gap-3">
										<NameAvatar
											name={customer.name}
											hue={hueOf(customer.id)}
											size="lg"
										/>
										<span className="min-w-0">
											<Link
												to={buildOrgPath(
													organizationSlug,
													'/crm/customers/$customerId',
												)}
												params={{ customerId: customer.id }}
												className="block truncate font-medium outline-none before:absolute before:inset-0 focus-visible:before:ring-2 focus-visible:before:ring-ring focus-visible:before:ring-inset"
											>
												{customer.name}
											</Link>
											<span className="block truncate text-[12px] text-muted-foreground">
												{ACCOUNT_KIND_LABELS[kind]}
											</span>
										</span>
									</div>
								</TableCell>
								<TableCell className="max-xl:hidden">
									{customer.email || customer.phone ? (
										<>
											<span className="block truncate">
												{customer.email || customer.phone}
											</span>
											{customer.email && customer.phone && (
												<span className="block truncate text-[12px] text-muted-foreground">
													{customer.phone}
												</span>
											)}
										</>
									) : (
										<Dash />
									)}
								</TableCell>
								<TableCell className="text-right">
									{outstanding && outstanding.cents > 0 ? (
										<>
											<Money
												cents={outstanding.cents}
												className={cn(
													'block',
													late && 'font-medium text-state-alert',
												)}
											/>
											{late && outstanding.oldestDueAt && (
												<span className="block text-[12px] text-state-alert">
													échu depuis le {formatDate(outstanding.oldestDueAt)}
												</span>
											)}
										</>
									) : (
										<Dash label="Aucun encours" />
									)}
								</TableCell>
								<TableCell className="max-lg:hidden">
									<StageBadge stage={customer.pipeline_stage}>
										{customerPipelineStageLabel(customer.pipeline_stage)}
									</StageBadge>
								</TableCell>
								<TableCell className="text-muted-foreground max-lg:hidden">
									{formatDate(customer.created_at)}
								</TableCell>
								<TableCell className="pr-4">
									<AccountStatusBadge
										status={accountStatusOf(customer.status)}
									/>
								</TableCell>
							</TableRow>
						)
					})}
				</TableBody>
			</Table>
		</div>
	)
}
