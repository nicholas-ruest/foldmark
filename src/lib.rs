//! Booklet imposition for saddle-stitched signatures.
//!
//! A printed sheet contributes four booklet pages. [`plan_booklet`] divides a
//! document into signatures, pads incomplete signatures with explicit blanks,
//! and returns the page positions for both sides of every physical sheet.

use serde::Serialize;
use std::fmt;

/// Reading direction of the finished booklet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Binding {
    /// The spine is on the left, as in most left-to-right books.
    Left,
    /// The spine is on the right; every imposed spread is mirrored.
    Right,
}

/// A logical document page or a padding position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum Page {
    Number(u32),
    #[serde(rename = "blank")]
    Blank,
}

impl fmt::Display for Page {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(number) => number.fmt(f),
            Self::Blank => f.write_str("blank"),
        }
    }
}

/// The two page positions visible on one side of a physical sheet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Side {
    pub left: Page,
    pub right: Page,
}

/// The imposed pages for one physical sheet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Sheet {
    /// One-based sheet number within its signature, from outside to inside.
    pub number: u32,
    pub front: Side,
    pub back: Side,
}

/// One independently folded and nested bundle of sheets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Signature {
    pub number: u32,
    pub first_page: u32,
    pub last_page: u32,
    pub padded_pages: u32,
    pub blank_positions: u32,
    pub sheets: Vec<Sheet>,
}

/// A complete booklet plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BookletPlan {
    pub document_pages: u32,
    pub total_sheets: u32,
    pub total_blank_positions: u32,
    pub signatures: Vec<Signature>,
}

/// Invalid planner input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanError {
    EmptyDocument,
    EmptySignature,
    ArithmeticOverflow,
}

impl fmt::Display for PlanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyDocument => f.write_str("document must contain at least one page"),
            Self::EmptySignature => f.write_str("signature must contain at least one sheet"),
            Self::ArithmeticOverflow => f.write_str("page count is too large to impose safely"),
        }
    }
}

impl std::error::Error for PlanError {}

/// Plan the sheet order for a saddle-stitched booklet.
///
/// `signature_sheets` is the maximum number of physical sheets nested in one
/// signature. A value of `4`, for example, limits each signature to 16 pages.
pub fn plan_booklet(
    document_pages: u32,
    signature_sheets: u32,
    binding: Binding,
) -> Result<BookletPlan, PlanError> {
    if document_pages == 0 {
        return Err(PlanError::EmptyDocument);
    }
    if signature_sheets == 0 {
        return Err(PlanError::EmptySignature);
    }

    let capacity = signature_sheets
        .checked_mul(4)
        .ok_or(PlanError::ArithmeticOverflow)?;
    let mut signatures = Vec::new();
    let mut first_page = 1_u32;
    let mut signature_number = 1_u32;
    let mut total_sheets = 0_u32;
    let mut total_blank_positions = 0_u32;

    while first_page <= document_pages {
        let remaining = document_pages - first_page + 1;
        let actual_pages = remaining.min(capacity);
        let padded_pages = round_up_to_four(actual_pages)?;
        let blank_positions = padded_pages - actual_pages;
        let last_page = first_page + actual_pages - 1;
        let sheet_count = padded_pages / 4;
        let mut sheets = Vec::with_capacity(sheet_count as usize);

        for sheet_index in 0..sheet_count {
            let low = sheet_index
                .checked_mul(2)
                .and_then(|value| value.checked_add(1))
                .ok_or(PlanError::ArithmeticOverflow)?;
            let high = padded_pages - sheet_index * 2;

            let mut front = Side {
                left: page_at(first_page, actual_pages, high),
                right: page_at(first_page, actual_pages, low),
            };
            let mut back = Side {
                left: page_at(first_page, actual_pages, low + 1),
                right: page_at(first_page, actual_pages, high - 1),
            };

            if binding == Binding::Right {
                mirror(&mut front);
                mirror(&mut back);
            }

            sheets.push(Sheet {
                number: sheet_index + 1,
                front,
                back,
            });
        }

        total_sheets = total_sheets
            .checked_add(sheet_count)
            .ok_or(PlanError::ArithmeticOverflow)?;
        total_blank_positions = total_blank_positions
            .checked_add(blank_positions)
            .ok_or(PlanError::ArithmeticOverflow)?;
        signatures.push(Signature {
            number: signature_number,
            first_page,
            last_page,
            padded_pages,
            blank_positions,
            sheets,
        });

        first_page = last_page
            .checked_add(1)
            .ok_or(PlanError::ArithmeticOverflow)?;
        signature_number = signature_number
            .checked_add(1)
            .ok_or(PlanError::ArithmeticOverflow)?;
    }

    Ok(BookletPlan {
        document_pages,
        total_sheets,
        total_blank_positions,
        signatures,
    })
}

fn round_up_to_four(value: u32) -> Result<u32, PlanError> {
    value
        .checked_add(3)
        .map(|rounded| rounded / 4 * 4)
        .ok_or(PlanError::ArithmeticOverflow)
}

fn page_at(first_page: u32, actual_pages: u32, local_page: u32) -> Page {
    if local_page > actual_pages {
        Page::Blank
    } else {
        Page::Number(first_page + local_page - 1)
    }
}

fn mirror(side: &mut Side) {
    std::mem::swap(&mut side.left, &mut side.right);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn imposes_one_sheet() {
        let plan = plan_booklet(4, 4, Binding::Left).unwrap();
        let sheet = &plan.signatures[0].sheets[0];
        assert_eq!(sheet.front.left, Page::Number(4));
        assert_eq!(sheet.front.right, Page::Number(1));
        assert_eq!(sheet.back.left, Page::Number(2));
        assert_eq!(sheet.back.right, Page::Number(3));
    }

    #[test]
    fn orders_outer_and_inner_sheets() {
        let plan = plan_booklet(8, 2, Binding::Left).unwrap();
        let sheets = &plan.signatures[0].sheets;
        assert_eq!(sheets[0].front, side(8, 1));
        assert_eq!(sheets[0].back, side(2, 7));
        assert_eq!(sheets[1].front, side(6, 3));
        assert_eq!(sheets[1].back, side(4, 5));
    }

    #[test]
    fn pads_incomplete_document_with_explicit_blanks() {
        let plan = plan_booklet(6, 4, Binding::Left).unwrap();
        assert_eq!(plan.total_sheets, 2);
        assert_eq!(plan.total_blank_positions, 2);
        assert_eq!(plan.signatures[0].sheets[0].front.left, Page::Blank);
        assert_eq!(plan.signatures[0].sheets[0].back.right, Page::Blank);
    }

    #[test]
    fn splits_document_at_signature_capacity() {
        let plan = plan_booklet(18, 2, Binding::Left).unwrap();
        assert_eq!(plan.signatures.len(), 3);
        assert_eq!(plan.total_sheets, 5);
        assert_eq!(plan.total_blank_positions, 2);
        assert_eq!(plan.signatures[0].first_page, 1);
        assert_eq!(plan.signatures[0].last_page, 8);
        assert_eq!(plan.signatures[1].first_page, 9);
        assert_eq!(plan.signatures[1].last_page, 16);
        assert_eq!(plan.signatures[2].first_page, 17);
        assert_eq!(plan.signatures[2].last_page, 18);
    }

    #[test]
    fn carries_global_page_numbers_into_later_signatures() {
        let plan = plan_booklet(12, 2, Binding::Left).unwrap();
        let sheet = &plan.signatures[1].sheets[0];
        assert_eq!(sheet.front, side(12, 9));
        assert_eq!(sheet.back, side(10, 11));
    }

    #[test]
    fn right_binding_mirrors_both_sides() {
        let plan = plan_booklet(8, 2, Binding::Right).unwrap();
        let sheet = &plan.signatures[0].sheets[0];
        assert_eq!(sheet.front, side(1, 8));
        assert_eq!(sheet.back, side(7, 2));
    }

    #[test]
    fn every_page_appears_exactly_once() {
        let plan = plan_booklet(37, 3, Binding::Left).unwrap();
        let mut pages = plan
            .signatures
            .iter()
            .flat_map(|signature| &signature.sheets)
            .flat_map(|sheet| {
                [
                    sheet.front.left,
                    sheet.front.right,
                    sheet.back.left,
                    sheet.back.right,
                ]
            })
            .filter_map(|page| match page {
                Page::Number(number) => Some(number),
                Page::Blank => None,
            })
            .collect::<Vec<_>>();
        pages.sort_unstable();
        assert_eq!(pages, (1..=37).collect::<Vec<_>>());
    }

    #[test]
    fn rejects_zero_pages() {
        assert_eq!(
            plan_booklet(0, 4, Binding::Left),
            Err(PlanError::EmptyDocument)
        );
    }

    #[test]
    fn rejects_zero_signature_sheets() {
        assert_eq!(
            plan_booklet(12, 0, Binding::Left),
            Err(PlanError::EmptySignature)
        );
    }

    #[test]
    fn rejects_capacity_overflow() {
        assert_eq!(
            plan_booklet(12, u32::MAX, Binding::Left),
            Err(PlanError::ArithmeticOverflow)
        );
    }

    fn side(left: u32, right: u32) -> Side {
        Side {
            left: Page::Number(left),
            right: Page::Number(right),
        }
    }
}
