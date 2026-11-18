import { Plus, X } from 'lucide-react'
import { Button } from '#/components/ui/button'
import { Input } from '#/components/ui/input'

export function optionFieldId(questionId: string, position: number): string {
	return `question-${questionId}-option-${position}`
}

export function addOptionButtonId(questionId: string): string {
	return `question-${questionId}-add-option`
}

export function QuestionOptions({
	questionId,
	options,
	multiple,
	readOnly,
	onAdd,
	onRename,
	onRemove,
}: {
	questionId: string
	options: readonly string[]
	multiple: boolean
	readOnly: boolean
	onAdd: () => void
	onRename: (position: number, value: string) => void
	onRemove: (position: number) => void
}) {
	if (options.length === 0) {
		return (
			<div className="rounded-lg border border-dashed bg-card px-4 py-6 text-center">
				<p className="text-muted-foreground text-sm">
					Aucune option pour le moment.
				</p>
				{readOnly ? null : (
					<Button
						id={addOptionButtonId(questionId)}
						type="button"
						variant="outline"
						size="sm"
						className="mt-3 min-h-11 sm:min-h-8"
						onClick={onAdd}
					>
						<Plus aria-hidden />
						Ajouter une option
					</Button>
				)}
			</div>
		)
	}
	return (
		<fieldset className="m-0 grid min-w-0 gap-2 border-0 p-0">
			<legend className="mb-2 font-medium text-sm">Options</legend>
			<ol className="grid gap-2">
				{options.map((option, position) => {
					const id = optionFieldId(questionId, position)
					return (
						<li key={id} className="flex items-center gap-2">
							<span
								aria-hidden
								className={
									multiple
										? 'size-4 shrink-0 rounded-sm border-2 border-muted-foreground/40'
										: 'size-4 shrink-0 rounded-full border-2 border-muted-foreground/40'
								}
							/>
							<label htmlFor={id} className="sr-only">
								Option {position + 1}
							</label>
							<Input
								id={id}
								value={option}
								placeholder={`Option ${position + 1}`}
								disabled={readOnly}
								className="min-h-11 sm:min-h-9"
								onChange={(event) => onRename(position, event.target.value)}
								onKeyDown={(event) => {
									if (
										event.key === 'Enter' &&
										position === options.length - 1 &&
										!readOnly
									) {
										event.preventDefault()
										onAdd()
									}
								}}
							/>
							{readOnly ? null : (
								<Button
									type="button"
									variant="ghost"
									size="icon"
									className="size-11 shrink-0 sm:size-9"
									aria-label={`Retirer l’option ${position + 1}`}
									onClick={() => onRemove(position)}
								>
									<X aria-hidden />
								</Button>
							)}
						</li>
					)
				})}
			</ol>
			{readOnly ? null : (
				<div>
					<Button
						id={addOptionButtonId(questionId)}
						type="button"
						variant="ghost"
						size="sm"
						className="min-h-11 sm:min-h-8"
						onClick={onAdd}
					>
						<Plus aria-hidden />
						Ajouter une option
					</Button>
				</div>
			)}
		</fieldset>
	)
}
