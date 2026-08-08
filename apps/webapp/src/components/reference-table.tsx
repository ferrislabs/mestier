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
			<div className="overflow-x-auto">
				<table className="w-full min-w-[720px] border-collapse text-sm">
					<thead>
						{table.getHeaderGroups().map((headerGroup) => (
							<tr key={headerGroup.id} className="border-b bg-muted/50">
								{headerGroup.headers.map((header) => (
									<th
										key={header.id}
										className="px-5 py-3 text-left text-xs font-semibold uppercase text-muted-foreground"
									>
										{header.isPlaceholder
											? null
											: flexRender(
													header.column.columnDef.header,
													header.getContext(),
												)}
									</th>
								))}
							</tr>
						))}
					</thead>
					<tbody>
						{table.getRowModel().rows.length === 0 ? (
							<tr>
								<td colSpan={columns.length} className="px-5 py-12 text-center">
									<div className="mx-auto flex max-w-sm flex-col items-center gap-2">
										<p className="font-medium">{emptyTitle}</p>
										<p className="text-sm text-muted-foreground">
											{emptyDescription}
										</p>
									</div>
								</td>
							</tr>
						) : (
							table.getRowModel().rows.map((row) => (
								<tr
									key={row.id}
									className="group border-b transition hover:bg-muted/35 hover:shadow-xs last:border-b-0"
								>
									{row.getVisibleCells().map((cell) => (
										<td key={cell.id} className="px-5 py-3 align-middle">
											{flexRender(
												cell.column.columnDef.cell,
												cell.getContext(),
											)}
										</td>
									))}
								</tr>
							))
						)}
					</tbody>
				</table>
			</div>
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
}

export function RowActions({
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
					<span className="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-xs font-medium text-muted-foreground">
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
			<p className="mt-0.5 truncate font-mono text-xs text-muted-foreground">
				id: {id}
			</p>
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
		<span className="font-medium tabular-nums">
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
