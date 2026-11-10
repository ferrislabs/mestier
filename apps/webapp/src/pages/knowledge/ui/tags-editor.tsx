import { X } from 'lucide-react'
import { useId } from 'react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'
import { Label } from '#/components/ui/label'

interface TagsEditorProps {
	tags: readonly string[]
	input: string
	disabled?: boolean
	onInputChange: (input: string) => void
	onAdd: () => void
	onRemove: (tag: string) => void
}

export function TagsEditor({
	tags,
	input,
	disabled = false,
	onInputChange,
	onAdd,
	onRemove,
}: TagsEditorProps) {
	const id = useId()
	return (
		<div className="grid gap-2">
			<Label htmlFor={id}>Étiquettes</Label>
			{tags.length > 0 ? (
				<ul className="flex flex-wrap gap-1.5">
					{tags.map((tag) => (
						<li
							key={tag}
							className="flex items-center gap-1 rounded-full bg-muted py-0.5 pr-1 pl-2.5 text-sm"
						>
							{tag}
							{disabled ? null : (
								<button
									type="button"
									aria-label={`Retirer l’étiquette ${tag}`}
									onClick={() => onRemove(tag)}
									className="rounded-full p-0.5 outline-none hover:bg-background focus-visible:ring-[3px] focus-visible:ring-ring/50"
								>
									<X aria-hidden className="size-3" />
								</button>
							)}
						</li>
					))}
				</ul>
			) : null}
			{disabled ? null : (
				<div className="flex gap-2">
					<Input
						id={id}
						value={input}
						placeholder="Ajouter une étiquette"
						autoComplete="off"
						onChange={(event) => onInputChange(event.target.value)}
						onKeyDown={(event) => {
							if (event.key === 'Enter' || event.key === ',') {
								event.preventDefault()
								onAdd()
							}
						}}
					/>
					<Button
						type="button"
						variant="outline"
						disabled={input.trim() === ''}
						onClick={onAdd}
					>
						Ajouter
					</Button>
				</div>
			)}
		</div>
	)
}
