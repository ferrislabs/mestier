import { Link } from '@tanstack/react-router'
import {
	Clock,
	Loader2,
	MoreHorizontal,
	Plus,
	Save,
	Search,
	Trash2,
	Undo2,
	Users,
} from 'lucide-react'
import type * as React from 'react'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'
import {
	MetricCard,
	PageHeader,
	PageShell,
	SectionCard,
	SectionHeader,
	StatusBadge,
} from '#/components/ui/surface'
import type { Employee } from '#/hooks/use-reference-catalog'
import type { EmployeeFormValues, EmployeeListData } from '#/pages/hr/types'

interface FormBinding<T> {
	values: T
	isPending: boolean
	onChange: (patch: Partial<T>) => void
	onSubmit: () => void
}

export interface EmployeeDraft {
	id: string
	values: EmployeeFormValues
}

interface EmployeeListUIProps {
	organizationName: string
	isLoading: boolean
	error: string | null
	data: EmployeeListData
	search: string
	onSearchChange: (value: string) => void
	createForm: FormBinding<EmployeeFormValues>
	draft: EmployeeDraft | null
	isSaving: boolean
	onEdit: (employee: Employee) => void
	onDraftChange: (values: EmployeeFormValues) => void
	onCancelEdit: () => void
	onSaveEdit: () => void
	onDeleteEmployee: (employee: Employee) => Promise<unknown>
}

export function EmployeeListUI({
	organizationName,
	isLoading,
	error,
	data,
	search,
	onSearchChange,
	createForm,
	draft,
	isSaving,
	onEdit,
	onDraftChange,
	onCancelEdit,
	onSaveEdit,
	onDeleteEmployee,
}: EmployeeListUIProps) {
	return (
		<PageShell>
			<PageHeader
				eyebrow={organizationName}
				title="Employés"
				description="Gérez les membres de l’équipe et leurs taux horaires."
			/>

			<MetricCard
				label="Employés"
				value={data.employees.length}
				hint="Taux horaires configurés"
				icon={<Users className="size-4" />}
			/>

			{error ? (
				<div className="rounded-lg border border-destructive/30 bg-destructive-soft px-4 py-3 text-sm text-destructive">
					{error}
				</div>
			) : null}

			<div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
				<div className="relative w-full lg:w-80">
					<Search className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground" />
					<Input
						type="search"
						value={search}
						onChange={(event) => onSearchChange(event.target.value)}
						placeholder="Rechercher un employé…"
						className="pl-9"
					/>
				</div>
			</div>

			<CreateEmployeeSection form={createForm} />

			{isLoading ? (
				<EmployeeListUI.Loading />
			) : (
				<EmployeeTable
					data={data.employees}
					draft={draft}
					isSaving={isSaving}
					onEdit={onEdit}
					onDraftChange={onDraftChange}
					onCancel={onCancelEdit}
					onSave={onSaveEdit}
					onDelete={onDeleteEmployee}
				/>
			)}
		</PageShell>
	)
}

EmployeeListUI.Loading = function EmployeeListLoading() {
	return (
		<PageShell>
			<SectionCard className="flex min-h-72 items-center justify-center gap-3 p-8 text-sm text-muted-foreground">
				<Loader2 className="size-5 animate-spin" />
				Chargement des employés…
			</SectionCard>
		</PageShell>
	)
}

interface CreateEmployeeSectionProps {
	form: FormBinding<EmployeeFormValues>
}

function CreateEmployeeSection({ form }: CreateEmployeeSectionProps) {
	return (
		<SectionCard>
			<SectionHeader
				title="Ajouter un employé"
				description="Les montants sont saisis en euros et stockés en centimes côté API."
			/>
			<div className="grid grid-cols-1 gap-4 p-5 lg:grid-cols-[1fr_auto] lg:items-end">
				<div className="grid grid-cols-1 gap-4 md:grid-cols-3">
					<TextField
						label="Nom"
						value={form.values.name}
						onChange={(name) => form.onChange({ name })}
					/>
					<TextField
						label="Taux horaire"
						value={form.values.hourlyRate}
						onChange={(hourlyRate) => form.onChange({ hourlyRate })}
						inputMode="decimal"
						suffix="€/h"
					/>
					<TextField
						label="Compte Ferriskey"
						value={form.values.userId}
						onChange={(userId) => form.onChange({ userId })}
						placeholder="UUID optionnel"
					/>
				</div>
				<CreateButton isPending={form.isPending} onClick={form.onSubmit} />
			</div>
		</SectionCard>
	)
}

interface EmployeeTableProps {
	data: Employee[]
	draft: EmployeeDraft | null
	isSaving: boolean
	onEdit: (employee: Employee) => void
	onDraftChange: (values: EmployeeFormValues) => void
	onCancel: () => void
	onSave: () => void
	onDelete: (employee: Employee) => Promise<unknown>
}

function EmployeeTable({
	data,
	draft,
	isSaving,
	onEdit,
	onDraftChange,
	onCancel,
	onSave,
	onDelete,
}: EmployeeTableProps) {
	return (
		<SectionCard>
			<SectionHeader
				title={`Employés (${data.length})`}
				description="Taux horaires et rattachements aux comptes utilisateurs."
			/>
			<div className="overflow-x-auto">
				<table className="w-full min-w-[720px] border-collapse text-sm">
					<thead>
						<tr className="border-b bg-muted/50">
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								Employé
							</th>
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								Compte
							</th>
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								Taux
							</th>
							<th className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground">
								<span className="sr-only">Actions</span>
							</th>
						</tr>
					</thead>
					<tbody>
						{data.length === 0 ? (
							<tr>
								<td colSpan={4} className="px-5 py-12 text-center">
									<div className="mx-auto flex max-w-sm flex-col items-center gap-2">
										<p className="font-medium">Aucun employé trouvé</p>
										<p className="text-sm text-muted-foreground">
											Ajoutez un employé pour le rendre disponible dans les
											prochains workflows opérationnels.
										</p>
									</div>
								</td>
							</tr>
						) : (
							data.map((employee) => {
								const isEditing = draft?.id === employee.id
								return (
									<tr
										key={employee.id}
										className="group border-b transition hover:bg-muted/35 hover:shadow-xs last:border-b-0"
									>
										<td className="px-5 py-3 align-middle">
											{isEditing ? (
												<Input
													value={draft.values.name}
													onChange={(event) =>
														onDraftChange({
															...draft.values,
															name: event.target.value,
														})
													}
												/>
											) : (
												<RowIdentity title={employee.name} id={employee.id} />
											)}
										</td>
										<td className="px-5 py-3 align-middle">
											{isEditing ? (
												<Input
													value={draft.values.userId}
													onChange={(event) =>
														onDraftChange({
															...draft.values,
															userId: event.target.value,
														})
													}
													placeholder="UUID optionnel"
												/>
											) : employee.user_id ? (
												<StatusBadge tone="success">lié</StatusBadge>
											) : (
												<StatusBadge>non lié</StatusBadge>
											)}
										</td>
										<td className="px-5 py-3 align-middle">
											{isEditing ? (
												<Input
													value={draft.values.hourlyRate}
													onChange={(event) =>
														onDraftChange({
															...draft.values,
															hourlyRate: event.target.value,
														})
													}
													inputMode="decimal"
												/>
											) : (
												<MoneyCell
													value={employee.hourly_rate_cents}
													suffix="/h"
												/>
											)}
										</td>
										<td className="px-5 py-3 align-middle">
											<RowActions
												employeeId={employee.id}
												isEditing={isEditing}
												isSaving={isSaving}
												onEdit={() => onEdit(employee)}
												onCancel={onCancel}
												onSave={onSave}
												onDelete={() => onDelete(employee)}
											/>
										</td>
									</tr>
								)
							})
						)}
					</tbody>
				</table>
			</div>
		</SectionCard>
	)
}

interface RowActionsProps {
	employeeId: string
	isEditing: boolean
	isSaving: boolean
	onEdit: () => void
	onCancel: () => void
	onSave: () => void
	onDelete: () => void
}

function RowActions({
	employeeId,
	isEditing,
	isSaving,
	onEdit,
	onCancel,
	onSave,
	onDelete,
}: RowActionsProps) {
	if (isEditing) {
		return (
			<div className="flex justify-end gap-1">
				<Button size="icon-sm" variant="ghost" onClick={onCancel}>
					<Undo2 />
					<span className="sr-only">Annuler</span>
				</Button>
				<Button size="icon-sm" onClick={onSave} disabled={isSaving}>
					{isSaving ? <Loader2 className="animate-spin" /> : <Save />}
					<span className="sr-only">Enregistrer</span>
				</Button>
			</div>
		)
	}

	return (
		<div className="flex justify-end opacity-0 transition group-hover:opacity-100 group-focus-within:opacity-100">
			<DropdownMenu>
				<DropdownMenuTrigger asChild>
					<Button size="icon-sm" variant="ghost">
						<MoreHorizontal />
						<span className="sr-only">Actions</span>
					</Button>
				</DropdownMenuTrigger>
				<DropdownMenuContent align="end">
					<DropdownMenuItem onClick={onEdit}>Modifier</DropdownMenuItem>
					<DropdownMenuItem asChild>
						<Link
							to="/hr/employees/$employeeId/work-time"
							params={{ employeeId }}
						>
							<Clock />
							Temps de travail
						</Link>
					</DropdownMenuItem>
					<DropdownMenuSeparator />
					<DropdownMenuItem variant="destructive" onClick={onDelete}>
						<Trash2 />
						Supprimer
					</DropdownMenuItem>
				</DropdownMenuContent>
			</DropdownMenu>
		</div>
	)
}

interface TextFieldProps
	extends Omit<
		React.InputHTMLAttributes<HTMLInputElement>,
		'value' | 'onChange'
	> {
	label: string
	value: string
	onChange: (value: string) => void
	suffix?: string
}

function TextField({
	label,
	value,
	onChange,
	suffix,
	...props
}: TextFieldProps) {
	const id = label.toLowerCase().replace(/\s+/g, '-')
	return (
		<div className="flex flex-col gap-2">
			<Label htmlFor={id}>{label}</Label>
			<div className="relative">
				<Input
					id={id}
					value={value}
					onChange={(event) => onChange(event.target.value)}
					className={suffix ? 'pr-14' : undefined}
					{...props}
				/>
				{suffix ? (
					<span className="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-xs font-medium text-muted-foreground">
						{suffix}
					</span>
				) : null}
			</div>
		</div>
	)
}

function CreateButton({
	isPending,
	onClick,
}: {
	isPending: boolean
	onClick: () => void
}) {
	return (
		<Button onClick={onClick} disabled={isPending}>
			{isPending ? <Loader2 className="animate-spin" /> : <Plus />}
			Ajouter
		</Button>
	)
}

function RowIdentity({ title, id }: { title: string; id: string }) {
	return (
		<div className="min-w-0">
			<p className="truncate font-medium">{title}</p>
			<p className="mt-0.5 truncate font-mono text-xs text-muted-foreground">
				id: {id}
			</p>
		</div>
	)
}

function MoneyCell({
	value,
	suffix,
}: {
	/** `null` renders as "non renseigné" — an absent rate is not a free one. */
	value: number | null | undefined
	suffix: string
}) {
	if (value === null || value === undefined) {
		return <span className="text-muted-foreground italic">Non renseigné</span>
	}

	return (
		<span className="font-medium tabular-nums">
			{formatMoney(value)}
			<span className="text-muted-foreground">{suffix}</span>
		</span>
	)
}

function formatMoney(value: number): string {
	return new Intl.NumberFormat('fr-FR', {
		style: 'currency',
		currency: 'EUR',
	}).format(value / 100)
}
