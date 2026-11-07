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
import type { CustomerListData } from '#/pages/customers/lib/accounts'
import { formatDate } from '#/pages/customers/lib/format'
import {
	ACCOUNT_KIND_LABELS,
	accountStatusOf,
	kindOf,
} from '#/pages/customers/types'
import { hueOf, NameAvatar } from '#/pages/customers/ui/avatars'
import { Dash } from '#/pages/customers/ui/detail-card'
import { Money } from '#/pages/customers/ui/money'
import { AccountStatusBadge } from '#/pages/customers/ui/status-badges'

export function CustomersTable({
	organizationSlug,
	customers,
	dataOf,
}: {
	organizationSlug: string
	customers: readonly Customer[]
	dataOf: (customerId: string) => CustomerListData
}) {
	return (
		<div className="overflow-hidden rounded-lg border bg-card">
			<Table>
				<TableHeader>
					<TableRow className="hover:bg-transparent">
						<TableHead className="pl-4">Client</TableHead>
						<TableHead className="max-md:hidden">Ville</TableHead>
						<TableHead className="max-xl:hidden">Contact principal</TableHead>
						<TableHead className="text-right">Encours</TableHead>
						<TableHead className="text-right max-lg:hidden">
							CA 12 mois
						</TableHead>
						<TableHead className="max-lg:hidden">Dernière activité</TableHead>
						<TableHead className="pr-4">Statut</TableHead>
					</TableRow>
				</TableHeader>
				<TableBody>
					{customers.map((customer) => {
						const data = dataOf(customer.id)
						const archived = customer.status === 'ARCHIVED'
						const kind = kindOf(customer)
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
													'/crm/clients/$customerId',
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
								<TableCell className="max-md:hidden">
									{data.city || <Dash />}
								</TableCell>
								<TableCell className="max-xl:hidden">
									{data.contactName ? (
										<>
											<span className="block truncate">{data.contactName}</span>
											{data.contactRole && (
												<span className="block truncate text-[12px] text-muted-foreground">
													{data.contactRole}
												</span>
											)}
										</>
									) : (
										<Dash />
									)}
								</TableCell>
								<TableCell className="text-right">
									{data.outstanding > 0 ? (
										<>
											<Money
												cents={data.outstanding}
												className={cn(
													'block',
													data.overdue > 0 && 'font-medium text-state-alert',
												)}
											/>
											{data.overdue > 0 && (
												<span className="block text-[12px] text-state-alert">
													dont <Money cents={data.overdue} /> en retard
												</span>
											)}
										</>
									) : (
										<Dash label="Aucun encours" />
									)}
								</TableCell>
								<TableCell className="text-right max-lg:hidden">
									{data.revenue12m > 0 ? (
										<Money cents={data.revenue12m} />
									) : (
										<Dash label="Aucun chiffre d’affaires" />
									)}
								</TableCell>
								<TableCell className="text-muted-foreground max-lg:hidden">
									{data.lastActivity ? (
										formatDate(data.lastActivity)
									) : (
										<Dash label="Aucune activité" />
									)}
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
