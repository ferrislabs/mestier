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

pub(crate) fn ensure_known(graph: &Graph, answers: &Answers) -> Result<(), SurveyError> {
    match answers.keys().find(|id| graph.question(id).is_none()) {
        Some(id) => Err(SurveyError::AnswerUnknownQuestion(*id)),
        None => Ok(()),
    }
}

pub(crate) fn validate(graph: &Graph, answers: &Answers) -> Result<(), SurveyError> {
    for (id, answer) in answers {
        let node = graph
            .question(id)
            .ok_or(SurveyError::AnswerUnknownQuestion(*id))?;
        check_answer(node, answer)?;
    }
    Ok(())
}

fn check_answer(node: &QuestionNode, answer: &Answer) -> Result<(), SurveyError> {
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
        (QuestionKind::Photo, Answer::Photos(photos)) => check_photos(node, photos, id),
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

fn check_photos(node: &QuestionNode, photos: &[String], id: QuestionId) -> Result<(), SurveyError> {
    let limit = node.max_photos.map_or(DEFAULT_MAX_PHOTOS, usize::from);
    if photos.len() > limit {
        return Err(SurveyError::AnswerTooManyPhotos(id));
    }
    if photos.iter().any(|photo| photo.trim().is_empty()) {
        return Err(SurveyError::AnswerEmptyPhoto(id));
    }
    Ok(())
}
