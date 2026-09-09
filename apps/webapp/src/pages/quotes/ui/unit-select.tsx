import { Check, ChevronsUpDown, Plus } from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import { cn } from '#/lib/utils'
import { formatUnit, UNIT_GROUPS } from '../types'

interface UnitSelectProps {
	value: string
	onChange: (unit: string) => void
	/** An organization's own units (#449), offered alongside the ten
	 * built-in ones. Omitted where a caller has no use for them yet (a
	 * quote line inherits its unit from whatever it was built from, and a
	 * new one starts on a built-in default). */
	customUnits?: string[]
	/** When present, a search that matches nothing offers to create it as a
	 * new custom unit — only wired where that action makes sense (creating
	 * or editing a product/service in the catalogue, #449's own scope), not
	 * on every screen this picker appears on. */
	onCreateCustomUnit?: (code: string) => void
	isCreatingCustomUnit?: boolean
}

/**
 * The unit picker, shared by every screen that names a unit so they can
 * never offer different sets. A plain Radix `Select` cannot host a search
 * box inside its own popper without fighting its focus management, so this
 * is a `Popover` instead: a button showing the current value, and a filtered
 * list — grouped like before while idle, flattened once the search narrows it.
 */
export function UnitSelect({
	value,
	onChange,
	customUnits = [],
	onCreateCustomUnit,
	isCreatingCustomUnit,
}: UnitSelectProps) {
	const [open, setOpen] = useState(false)
	const [search, setSearch] = useState('')

	const normalizedSearch = search.trim().toLowerCase()
	const matchesSearch = (unit: string) =>
		!normalizedSearch ||
		unit.toLowerCase().includes(normalizedSearch) ||
		formatUnit(unit).toLowerCase().includes(normalizedSearch)

	const filteredGroups = UNIT_GROUPS.map((group) => ({
		...group,
		units: group.units.filter(matchesSearch),
	})).filter((group) => group.units.length > 0)
	const filteredCustomUnits = customUnits.filter(matchesSearch)

	const isKnownUnit =
		UNIT_GROUPS.some((group) =>
			group.units.some((unit) => unit.toLowerCase() === normalizedSearch),
		) || customUnits.some((unit) => unit.toLowerCase() === normalizedSearch)
	const canCreate =
		Boolean(onCreateCustomUnit) && normalizedSearch.length > 0 && !isKnownUnit

	const selectUnit = (unit: string) => {
		onChange(unit)
		setOpen(false)
		setSearch('')
	}

	return (
		<Popover
			open={open}
			onOpenChange={(next) => {
				setOpen(next)
				if (!next) setSearch('')
			}}
		>
			<PopoverTrigger asChild>
				<Button
					type="button"
					variant="outline"
					role="combobox"
					aria-expanded={open}
					className="w-full justify-between font-normal"
				>
					<span className="truncate">
						{value ? formatUnit(value) : 'Unité'}
					</span>
					<ChevronsUpDown className="size-4 shrink-0 text-muted-foreground" />
				</Button>
			</PopoverTrigger>
			<PopoverContent align="start" className="w-64 p-0">
				<div className="border-b p-2">
					<Input
						autoFocus
						value={search}
						onChange={(event) => setSearch(event.target.value)}
						placeholder="Rechercher une unité…"
					/>
				</div>
				<div className="max-h-64 overflow-y-auto p-1">
					{filteredGroups.map((group) => (
						<div key={group.label} className="mb-1 last:mb-0">
							<p className="px-2 py-1 text-xs font-medium text-muted-foreground">
								{group.label}
							</p>
							{group.units.map((unit) => (
								<UnitOption
									key={unit}
									unit={unit}
									selected={unit === value}
									onSelect={() => selectUnit(unit)}
								/>
							))}
						</div>
					))}
					{filteredCustomUnits.length > 0 ? (
						<div className="mb-1 last:mb-0">
							<p className="px-2 py-1 text-xs font-medium text-muted-foreground">
								Personnalisées
							</p>
							{filteredCustomUnits.map((unit) => (
								<UnitOption
									key={unit}
									unit={unit}
									selected={unit === value}
									onSelect={() => selectUnit(unit)}
								/>
							))}
						</div>
					) : null}
					{filteredGroups.length === 0 && filteredCustomUnits.length === 0 ? (
						<p className="px-2 py-3 text-center text-sm text-muted-foreground">
							Aucune unité trouvée
						</p>
					) : null}
					{canCreate ? (
						<button
							type="button"
							disabled={isCreatingCustomUnit}
							onClick={() => {
								onCreateCustomUnit?.(search.trim())
								selectUnit(search.trim())
							}}
							className="flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm text-primary transition-colors hover:bg-brand-soft disabled:cursor-not-allowed disabled:opacity-50"
						>
							<Plus className="size-3.5 shrink-0" />
							<span className="truncate">Créer « {search.trim()} »</span>
						</button>
					) : null}
				</div>
			</PopoverContent>
		</Popover>
	)
}

function UnitOption({
	unit,
	selected,
	onSelect,
}: {
	unit: string
	selected: boolean
	onSelect: () => void
}) {
	return (
		<button
			type="button"
			role="option"
			aria-selected={selected}
			onClick={onSelect}
			className={cn(
				'flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-sm transition-colors',
				selected
					? 'bg-brand-soft font-medium text-primary'
					: 'hover:bg-muted/60',
			)}
		>
			<Check className={cn('size-3.5 shrink-0', !selected && 'opacity-0')} />
			<span className="truncate">{formatUnit(unit)}</span>
		</button>
	)
}
