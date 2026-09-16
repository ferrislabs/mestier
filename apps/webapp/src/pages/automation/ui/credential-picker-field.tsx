import { Check, ChevronsUpDown, Plus } from 'lucide-react'
import { useState } from 'react'
import type { Schemas } from '#/api/api.client'
import { Button } from '#/components/ui/button'
import {
	Command,
	CommandEmpty,
	CommandGroup,
	CommandInput,
	CommandItem,
	CommandList,
} from '#/components/ui/command'
import { Field } from '#/components/ui/field'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import { cn } from '#/lib/utils'

export interface CredentialPickerFieldProps {
	label: string
	htmlFor: string
	credentials: Schemas.CredentialResponse[]
	value: string | null
	error: string | null
	onChange: (credentialId: string | null) => void
	onCreateNew: () => void
}

export function CredentialPickerField({
	label,
	htmlFor,
	credentials,
	value,
	error,
	onChange,
	onCreateNew,
}: CredentialPickerFieldProps) {
	const [open, setOpen] = useState(false)
	const selected =
		credentials.find((credential) => credential.id === value) ?? null

	function createNew() {
		setOpen(false)
		onCreateNew()
	}

	return (
		<Field label={label} htmlFor={htmlFor}>
			<Popover open={open} onOpenChange={setOpen}>
				<PopoverTrigger asChild>
					<Button
						id={htmlFor}
						type="button"
						variant="outline"
						role="combobox"
						aria-expanded={open}
						aria-invalid={error !== null}
						className="w-full justify-between font-normal"
					>
						<span
							className={cn(
								'truncate',
								selected === null && 'text-muted-foreground',
							)}
						>
							{selected
								? selected.name
								: credentials.length === 0
									? 'Aucune identification'
									: 'Choisir une identification…'}
						</span>
						<ChevronsUpDown className="size-4 shrink-0 opacity-50" />
					</Button>
				</PopoverTrigger>
				<PopoverContent align="start" className="w-72 p-0">
					<Command>
						<CommandInput
							autoFocus
							placeholder="Rechercher une identification…"
						/>
						<CommandList>
							<CommandEmpty>
								<div className="flex flex-col items-center gap-2 px-2 py-4">
									<p className="text-center text-sm text-muted-foreground">
										{credentials.length === 0
											? 'Aucune identification disponible'
											: 'Aucun résultat'}
									</p>
									<Button type="button" size="sm" onClick={createNew}>
										<Plus className="size-4" />
										Créer une identification
									</Button>
								</div>
							</CommandEmpty>
							<CommandGroup>
								{credentials.map((credential) => (
									<CommandItem
										key={credential.id}
										value={credential.name}
										onSelect={() => {
											onChange(credential.id)
											setOpen(false)
										}}
									>
										<Check
											className={cn(
												'size-4',
												credential.id === value ? 'opacity-100' : 'opacity-0',
											)}
										/>
										{credential.name}
									</CommandItem>
								))}
							</CommandGroup>
						</CommandList>
						{credentials.length > 0 ? (
							<div className="border-t p-1">
								<Button
									type="button"
									variant="ghost"
									size="sm"
									className="w-full justify-start"
									onClick={createNew}
								>
									<Plus className="size-4" />
									Nouvelle identification
								</Button>
							</div>
						) : null}
					</Command>
				</PopoverContent>
			</Popover>
			{error ? (
				<p role="alert" className="text-sm text-destructive">
					{error}
				</p>
			) : null}
		</Field>
	)
}
