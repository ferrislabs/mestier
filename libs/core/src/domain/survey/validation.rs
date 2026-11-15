use super::error::SurveyError;
use super::graph::{Answers, Graph, QuestionNode};
use super::value::{Answer, QuestionId, QuestionKind};

const DEFAULT_MAX_PHOTOS: usize = 5;

pub(crate) fn check_finite(answers: &Answers) -> Result<(), SurveyError> {
    for (id, answer) in answers {
        if matches!(answer, Answer::Number(number) if !number.is_finite()) {
            return Err(SurveyError::AnswerNotFinite(*id));
        }
    }
    Ok(())
}

pub(crate) fn retain_known(graph: &Graph, answers: &mut Answers) {
    answers.retain(|id, _| graph.question(id).is_some());
}

pub(crate) fn validate(
    graph: &Graph,
    answers: &Answers,
    photo_directory: &str,
) -> Result<(), SurveyError> {
    for (id, answer) in answers {
        let node = graph
            .question(id)
            .ok_or(SurveyError::AnswerUnknownQuestion(*id))?;
        check_answer(node, answer, photo_directory)?;
    }
    Ok(())
}

fn check_answer(
    node: &QuestionNode,
    answer: &Answer,
    photo_directory: &str,
) -> Result<(), SurveyError> {
    let id = node.id;
    match (node.kind, answer) {
        (QuestionKind::Text | QuestionKind::LongText, Answer::Text(_))
        | (QuestionKind::Date, Answer::Date(_))
        | (QuestionKind::Datetime, Answer::DateTime(_))
        | (QuestionKind::Phone, Answer::Phone(_))
        | (QuestionKind::Email, Answer::Email(_)) => Ok(()),
        (QuestionKind::Number, Answer::Number(number)) => {
            if number.is_finite() {
                Ok(())
            } else {
                Err(SurveyError::AnswerNotFinite(id))
            }
        }
        (QuestionKind::SingleChoice, Answer::Choice(option)) => known_option(node, option),
        (QuestionKind::MultipleChoice, Answer::Choices(options)) => {
            for (position, option) in options.iter().enumerate() {
                known_option(node, option)?;
                if options[..position].contains(option) {
                    return Err(SurveyError::AnswerDuplicateOption(id));
                }
            }
            Ok(())
        }
        (QuestionKind::Photo, Answer::Photos(photos)) => {
            check_photos(node, photos, id, photo_directory)
        }
        _ => Err(SurveyError::AnswerKindMismatch(id)),
    }
}

fn known_option(node: &QuestionNode, option: &str) -> Result<(), SurveyError> {
    if node.options.iter().any(|known| known == option) {
        Ok(())
    } else {
        Err(SurveyError::AnswerUnknownOption(node.id))
    }
}

fn check_photos(
    node: &QuestionNode,
    photos: &[String],
    id: QuestionId,
    photo_directory: &str,
) -> Result<(), SurveyError> {
    let limit = node.max_photos.map_or(DEFAULT_MAX_PHOTOS, usize::from);
    if photos.len() > limit {
        return Err(SurveyError::AnswerTooManyPhotos(id));
    }
    if photos.iter().any(|photo| photo.trim().is_empty()) {
        return Err(SurveyError::AnswerEmptyPhoto(id));
    }
    if photos
        .iter()
        .any(|photo| !stored_under(photo_directory, photo))
    {
        return Err(SurveyError::AnswerForeignPhoto(id));
    }
    Ok(())
}

fn is_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment != "."
        && segment != ".."
        && segment
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

fn is_root_segment(segment: &str) -> bool {
    is_segment(segment) && !segment.starts_with("survey-") && !segment.starts_with("library-")
}

pub(crate) fn stored_under(directory: &str, key: &str) -> bool {
    let Some((parent, name)) = key.rsplit_once('/') else {
        return false;
    };
    if !is_segment(name) {
        return false;
    }
    if parent == directory {
        return true;
    }
    parent
        .strip_suffix(directory)
        .and_then(|root| root.strip_suffix('/'))
        .is_some_and(|root| root.split('/').all(is_root_segment))
}
