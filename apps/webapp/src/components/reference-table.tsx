import {
	type ColumnDef,
	flexRender,
	getCoreRowModel,
	useReactTable,
} from '@tanstack/react-table'
import {
	Loader2,
	MoreHorizontal,
	Plus,
	Save,
	Trash2,
	Undo2,
} from 'lucide-react'
import type * as React from 'react'
import { DeveloperModeOnly } from '#/components/developer-mode-only'
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
import { SectionCard, SectionHeader } from '#/components/ui/surface'
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from '#/components/ui/table'
import type { PermissionName } from '#/hooks/use-permissions'
import { usePermissions } from '#/hooks/use-permissions'
import { cn } from '#/lib/utils'

export interface FormBinding<T> {
	values: T
	isPending: boolean
	onChange: (patch: Partial<T>) => void
	onSubmit: () => void
}

export interface ReferenceTableProps<T> {
	title: string
	description: string
	emptyTitle: string
	emptyDescription: string
	data: T[]
	columns: ColumnDef<T>[]
}

export function ReferenceTable<T>({
	title,
	description,
	emptyTitle,
	emptyDescription,
	data,
	columns,
}: ReferenceTableProps<T>) {
	const table = useReactTable({
		data,
		columns,
		getCoreRowModel: getCoreRowModel(),
	})

	return (
		<SectionCard>
			<SectionHeader
				title={`${title} (${data.length})`}
				description={description}
			/>
			<Table className="min-w-[720px]">
				<TableHeader>
					{table.getHeaderGroups().map((headerGroup) => (
						<TableRow key={headerGroup.id} className="hover:bg-transparent">
							{headerGroup.headers.map((header, index) => (
								<TableHead
									key={header.id}
									className={cn(
										'text-muted-foreground',
										index === 0 ? 'pl-5' : null,
										index === headerGroup.headers.length - 1 ? 'pr-5' : null,
									)}
								>
									{header.isPlaceholder
										? null
										: flexRender(
												header.column.columnDef.header,
												header.getContext(),
											)}
								</TableHead>
							))}
						</TableRow>
					))}
				</TableHeader>
				<TableBody>
					{table.getRowModel().rows.length === 0 ? (
						<TableRow className="hover:bg-transparent">
							<TableCell colSpan={columns.length} className="px-5 py-12">
								<div className="mx-auto flex max-w-sm flex-col items-center gap-1 text-center">
									<p className="font-medium">{emptyTitle}</p>
									<p className="whitespace-normal text-[13px] text-muted-foreground">
										{emptyDescription}
									</p>
								</div>
							</TableCell>
						</TableRow>
					) : (
						table.getRowModel().rows.map((row) => (
							<TableRow key={row.id} className="group">
								{row.getVisibleCells().map((cell, index) => (
									<TableCell
										key={cell.id}
										className={cn(
											'py-3',
											index === 0 ? 'pl-5' : null,
											index === row.getVisibleCells().length - 1
												? 'pr-5'
												: null,
										)}
									>
										{flexRender(cell.column.columnDef.cell, cell.getContext())}
									</TableCell>
								))}
							</TableRow>
						))
					)}
				</TableBody>
			</Table>
		</SectionCard>
	)
}

export interface RowActionsProps {
	isEditing: boolean
	isSaving: boolean
	onEdit: () => void
	onCancel: () => void
	onSave: () => void
	onDelete: () => void
	/**
	 * Hides the whole control — edit and delete are this row's only actions,
	 * so there is nothing left to show a trigger for without it — when the
	 * caller lacks this bit. Omit where the backend does not yet enforce an
	 * equivalent check: hiding a control the API would still accept is a
	 * false sense of security, not a real one (#403, #394's "inverse gap").
	 */
	permission?: PermissionName
}

/**
 * Splits on `permission` at this outer level, rather than calling
 * `usePermissions` unconditionally inside one component: that hook needs
 * `ActiveOrganizationProvider` in the tree, and most reuse sites (and their
 * tests) predate #403 and have no reason to carry that context when they
 * never pass a `permission`. Conditionally rendering one of two components
 * is fine where conditionally calling a hook inside one is not.
 */
export function RowActions({ permission, ...rest }: RowActionsProps) {
	if (permission) {
		return <GatedRowActions {...rest} permission={permission} />
	}
	return <RowActionsView {...rest} />
}

function GatedRowActions({
	permission,
	...rest
}: Omit<RowActionsProps, 'permission'> & { permission: PermissionName }) {
	const permissions = usePermissions()
	if (!permissions.data?.data.permissions.includes(permission)) return null
	return <RowActionsView {...rest} />
}

function RowActionsView({
	isEditing,
	isSaving,
	onEdit,
	onCancel,
	onSave,
	onDelete,
}: Omit<RowActionsProps, 'permission'>) {
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

export interface TextFieldProps
	extends Omit<
		React.InputHTMLAttributes<HTMLInputElement>,
		'value' | 'onChange'
	> {
	label: string
	value: string
	onChange: (value: string) => void
	suffix?: string
}

export function TextField({
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
					<span className="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 font-medium text-[12px] text-muted-foreground">
						{suffix}
					</span>
				) : null}
			</div>
		</div>
	)
}

export function CreateButton({
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

export function RowIdentity({ title, id }: { title: string; id: string }) {
	return (
		<div className="min-w-0">
			<p className="truncate font-medium">{title}</p>
			<DeveloperModeOnly>
				<p className="mt-0.5 truncate font-mono text-xs text-muted-foreground">
					id: {id}
				</p>
			</DeveloperModeOnly>
		</div>
	)
}

export function MoneyCell({
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
		<span className="tabular font-medium">
			{formatMoney(value)}
			<span className="text-muted-foreground">{suffix}</span>
		</span>
	)
}

export function centsToEuros(value: number): string {
	return (value / 100).toFixed(2).replace('.', ',')
}

export function formatMoney(value: number): string {
	return new Intl.NumberFormat('fr-FR', {
		style: 'currency',
		currency: 'EUR',
	}).format(value / 100)
}
