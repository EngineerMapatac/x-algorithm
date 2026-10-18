use crate::models::candidate::PostCandidate;
use crate::models::candidate_features::TopicFilteringExperiment;
use crate::models::query::ScoredPostsQuery;
use std::collections::HashMap;
use std::collections::HashSet;
use tracing::warn;
use xai_candidate_pipeline::filter::{Filter, FilterResult};
use xai_stats_receiver::global_stats_receiver;

const EXCLUDED_TOPIC_METRIC: &str = "TopicIdsFilter.excluded_topic_id";

pub struct TopicIdsFilter;

impl Filter<ScoredPostsQuery, PostCandidate> for TopicIdsFilter {
    fn enable(&self, query: &ScoredPostsQuery) -> bool {
        query.is_topic_request() || query.has_excluded_topics()
    }

    fn filter(
        &self,
        query: &ScoredPostsQuery,
        candidates: Vec<PostCandidate>,
    ) -> FilterResult<PostCandidate> {
        let (mut kept, mut removed) = if query.is_topic_request() {
            let query_topic_ids: HashSet<i64> = query.topic_ids.iter().copied().collect();
            let expanded = TopicIdExpansion::expand(&query_topic_ids);

            if query.is_bulk_topic_request() {
                let all_topic_ids = TopicIdExpansion::all_production_topic_ids();
                let excluded: HashSet<i64> = all_topic_ids.difference(&expanded).copied().collect();

                let (kept, removed): (Vec<_>, Vec<_>) =
                    candidates
                        .into_iter()
                        .partition(|c| match &c.filtered_topic_ids {
                            Some(candidate_topics) if !candidate_topics.is_empty() => {
                                !candidate_topics.iter().all(|tid| excluded.contains(tid))
                            }
                            _ => true,
                        });
                (kept, removed)
            } else {
                let supertopic_expanded: HashMap<i64, HashSet<i64>> = query
                    .topic_ids
                    .iter()
                    .map(|&tid| (tid, TopicIdExpansion::expand_supertopic(tid)))
                    .collect();

                let (kept, removed): (Vec<_>, Vec<_>) = candidates.into_iter().partition(|c| {
                    let exp_topics = c.filtered_topic_ids.as_deref().unwrap_or(&[]);
                    let unf_topics = c.unfiltered_topic_ids.as_deref().unwrap_or(&[]);

                    query.topic_ids.iter().any(|&tid| {
                        let tid_expanded = match TopicIdExpansion::category_ids(tid) {
                            Some(ids) => ids,
                            None => std::slice::from_ref(&tid),
                        };

                        let path1 = exp_topics.iter().any(|et| tid_expanded.contains(et));

                        if path1 {
                            return true;
                        }

                        if let Some(st_ids) = supertopic_expanded.get(&tid) {
                            let unf_has_topic =
                                unf_topics.iter().any(|ut| tid_expanded.contains(ut));
                            let exp_has_supertopic =
                                exp_topics.iter().any(|et| st_ids.contains(et));
                            unf_has_topic && exp_has_supertopic
                        } else {
                            false
                        }
                    })
                });
                (kept, removed)
            }
        } else {
            (candidates, vec![])
        };

        if query.has_excluded_topics() {
            if let Some(receiver) = global_stats_receiver() {
                for &tid in &query.excluded_topic_ids {
                    let tid_str = tid.to_string();
                    receiver.incr(EXCLUDED_TOPIC_METRIC, &[("topic_id", &tid_str)], 1);
                }
            }
            let excluded_ids: HashSet<i64> = query.excluded_topic_ids.iter().copied().collect();
            let mut excluded_expanded = TopicIdExpansion::expand(&excluded_ids);
            excluded_expanded.extend(&excluded_ids);

            let (new_kept, new_removed): (Vec<_>, Vec<_>) =
                kept.into_iter().partition(|c| match &c.filtered_topic_ids {
                    Some(candidate_topics) if !candidate_topics.is_empty() => !candidate_topics
                        .iter()
                        .any(|tid| excluded_expanded.contains(tid)),
                    _ => false,
                });
            kept = new_kept;
            removed.extend(new_removed);
        }

        FilterResult { kept, removed }
    }
}

pub struct TopicIdExpansion;

impl TopicIdExpansion {
    const SCIENCE_TECHNOLOGY: i64 = 1000000000000000001;
    const ENTERTAINMENT: i64 = 1000000000000000002;
    const BUSINESS_FINANCE: i64 = 1000000000000000003;
    const SPORTS: i64 = 1000000000000000004;

    pub const XAI_POLITICS: i64 = 1925952771733262336;
    pub const XAI_AI: i64 = 1925953013547450368;
    pub const XAI_GAMING: i64 = 1925949766673797120;
    pub const XAI_CRYPTOCURRENCY: i64 = 1925949693290295298;
    pub const XAI_US_IRAN_WAR: i64 = 2000000000000000001;
    pub const XAI_STOCKS_ECONOMY: i64 = 2000000000000000002;

    pub const XAI_NEWS: i64 = 1925949634972626944;
    const XAI_NATURAL_DISASTERS: i64 = 1925952795854745601;
    pub const XAI_MOVIES_TV: i64 = 1925949788068909057;
    const XAI_STREAMING: i64 = 1925950455806369792;
    pub const XAI_MUSIC: i64 = 1925949604224196608;
    pub const XAI_DANCE: i64 = 1925950383899267072;
    pub const XAI_CELEBRITY: i64 = 1925949555071176704;
    pub const XAI_ANIME: i64 = 1925949503837798401;
    const XAI_SOCCER: i64 = 1925950498420469760;
    const XAI_BUNDESLIGA: i64 = 1925951257107206144;
    const XAI_LA_LIGA: i64 = 1925951079100928001;
    const XAI_LIGUE1: i64 = 1925951305895272448;
    const XAI_UEFA_CL: i64 = 1925951334777233408;
    const XAI_UEFA_EL: i64 = 1925951366175813632;
    const XAI_UEFA_ECL: i64 = 1925951417010753536;
    const XAI_BASKETBALL: i64 = 1925950530842464256;
    const XAI_NBA: i64 = 1925951454121967616;
    const XAI_WNBA: i64 = 1925951561211023361;
    const XAI_AMERICAN_FOOTBALL: i64 = 1925950554133446656;
    const XAI_NCAA_FOOTBALL: i64 = 1925951645872975873;
    const XAI_NFL: i64 = 1925951614067650560;
    const XAI_BASEBALL: i64 = 1925950576740675584;
    const XAI_MLB: i64 = 1925951672561319936;
    const XAI_TENNIS: i64 = 1925950625835008001;
    const XAI_CRICKET: i64 = 1925950653035151360;
    const XAI_IPL: i64 = 1925951866464006145;
    const XAI_MMA: i64 = 1925950600140718080;
    const XAI_WWE: i64 = 1925950864117702656;
    const XAI_BOXING: i64 = 1925950713982537888;
    const XAI_GOLF: i64 = 1925950690574180352;
    const XAI_RACING: i64 = 1943810034309246976;
    const XAI_FORMULA1: i64 = 1925950746756800513;
    const XAI_NASCAR: i64 = 1925951703007858688;
    const XAI_ICE_HOCKEY: i64 = 1925950813043662848;
    const XAI_NHL: i64 = 1925951775544061952;
    const XAI_OLYMPICS: i64 = 1943811470724153345;
    const XAI_RUGBY: i64 = 1925950895595876352;
    const XAI_CYCLING: i64 = 1925950926910582786;
    const XAI_SKIING: i64 = 1925950954962096130;
    const XAI_SNOWBOARDING: i64 = 1925950982346682368;
    const XAI_HEALTH_FITNESS: i64 = 1925949856834588672;
    const XAI_NUTRITION: i64 = 1925953366091350017;
    const XAI_WORKOUTS: i64 = 1925953337020555264;
    const XAI_TRAVEL: i64 = 1925949812844769280;
    const XAI_AVIATION: i64 = 1925950031476957184;
    const XAI_FOOD: i64 = 1925949835649126400;
    const XAI_COOKING: i64 = 1925953199694819328;
    const XAI_BAKING: i64 = 1925953282146451456;
    const XAI_RESTAURANTS: i64 = 1925953243223339009;
    const XAI_DRINKS: i64 = 1925953310894252033;
    const XAI_MEMES: i64 = 1925949876694556672;
    const XAI_BEAUTY: i64 = 1925950199861538818;
    const XAI_FASHION: i64 = 1925949932181082112;
    const XAI_NATURE_OUTDOORS: i64 = 1925950228047216640;
    const XAI_PETS: i64 = 1925950249245282304;
    const XAI_CATS: i64 = 1925953794732433408;
    const XAI_DOGS: i64 = 1925953826281902080;
    const XAI_HOME_GARDEN: i64 = 1925950341452845057;
    const XAI_ART: i64 = 1925949898106540032;
    const XAI_RELIGION: i64 = 1925949956008878080;
    const XAI_SHOPPING: i64 = 1925949979966701568;
    const XAI_EDUCATION: i64 = 1925950405709631490;
    const XAI_CAREER: i64 = 1925950361245786112;
    const XAI_CARS: i64 = 1925950007624028160;
    const XAI_MOTORCYCLES: i64 = 1925950100397793280;
    const XAI_RELATIONSHIPS: i64 = 1925950276835454977;
    const XAI_FAMILY: i64 = 1925954059338493952;
    const XAI_MARRIAGE: i64 = 1925953930002907137;
    const XAI_PARENTING: i64 = 1925954095342338048;
    const XAI_PODCASTS: i64 = 1925950433807175680;
    const XAI_POP: i64 = 1925952179791151105;
    const XAI_K_POP: i64 = 1925952056088530944;
    const XAI_COUNTRY_MUSIC: i64 = 1925952353485602816;
    const XAI_ELECTRONIC: i64 = 1925952257893257217;
    const XAI_J_POP: i64 = 1925952123960709120;
    const XAI_ROCK: i64 = 1925952228818354177;
    const XAI_PHOTOGRAPHY: i64 = 1925954143472017408;
    const XAI_DESIGN: i64 = 1925954681496322048;
    const XAI_SOFTWARE_DEVELOPMENT: i64 = 1925953040130953216;
    const XAI_ROBOTICS: i64 = 1925953169307189248;
    const XAI_SPACE: i64 = 1925953141452718081;
    const XAI_BIOTECH: i64 = 1925953107038519296;
    const XAI_ELECTRONICS: i64 = 1925953073983139840;
    const XAI_STOCKS: i64 = 1925952876284727296;
    const XAI_ENTREPRENEURSHIP: i64 = 1925952916411564032;
    const XAI_REAL_ESTATE: i64 = 1925952947197784064;
    const XAI_PERSONAL_FINANCE: i64 = 1925952988486447104;
    const XAI_CRIME: i64 = 1925952744239620096;
    const XAI_DATING: i64 = 1925953884108824576;
    const XAI_ELECTIONS: i64 = 1925952820714332161;
    const XAI_ESPORTS: i64 = 1925950783448576000;
    const XAI_HIP_HOP: i64 = 1925952203962896384;
    const XAI_JAZZ: i64 = 1925952300213768192;
    const XAI_CONCERTS: i64 = 1925952145615974400;
    const XAI_MENTAL_HEALTH: i64 = 1925953421338726400;
    const XAI_MOTO_GP: i64 = 1925951732204335105;
    const XAI_PREMIER_LEAGUE: i64 = 1925951020036751361;
    const XAI_SERIE_A: i64 = 1925951109660577792;
    const XAI_CHRISTIANITY: i64 = 1925953452644978688;
    const XAI_BUDDHISM: i64 = 1925953574929842176;
    const XAI_HINDUISM: i64 = 1925953483758338048;
    const XAI_ISLAM: i64 = 1925953516834590721;
    const XAI_JUDAISM: i64 = 1925953757117865984;
    const XAI_DIGITAL_ART: i64 = 1925954197326831616;

    pub const XAI_SCIENCE: i64 = 1925949744683114496;
    pub const XAI_TECHNOLOGY: i64 = 1925949722688126976;
    pub const XAI_BUSINESS_FINANCE_REAL: i64 = 1925949659857530880;
    pub const XAI_SPORTS_REAL: i64 = 1925949452197478400;
    const XAI_ATHLETICS: i64 = 1925950838414942210;

    pub fn category_ids(topic_id: i64) -> Option<&'static [i64]> {
        match topic_id {
            Self::SCIENCE_TECHNOLOGY => Some(&[
                Self::XAI_SCIENCE,
                Self::XAI_TECHNOLOGY,
                Self::XAI_AI,
                Self::XAI_SOFTWARE_DEVELOPMENT,
                Self::XAI_ROBOTICS,
                Self::XAI_SPACE,
                Self::XAI_BIOTECH,
                Self::XAI_ELECTRONICS,
            ]),
            Self::ENTERTAINMENT => Some(&[
                Self::XAI_MOVIES_TV,
                Self::XAI_STREAMING,
                Self::XAI_MUSIC,
                Self::XAI_DANCE,
                Self::XAI_CELEBRITY,
                Self::XAI_GAMING,
                Self::XAI_ANIME,
            ]),
            Self::BUSINESS_FINANCE => Some(&[
                Self::XAI_BUSINESS_FINANCE_REAL,
                Self::XAI_CRYPTOCURRENCY,
                Self::XAI_STOCKS,
                Self::XAI_ENTREPRENEURSHIP,
                Self::XAI_REAL_ESTATE,
                Self::XAI_PERSONAL_FINANCE,
            ]),
            Self::SPORTS => Some(&[
                Self::XAI_SPORTS_REAL,
                Self::XAI_ATHLETICS,
                Self::XAI_SOCCER,
                Self::XAI_BUNDESLIGA,
                Self::XAI_LA_LIGA,
                Self::XAI_LIGUE1,
                Self::XAI_UEFA_CL,
                Self::XAI_UEFA_EL,
                Self::XAI_UEFA_ECL,
                Self::XAI_BASKETBALL,
                Self::XAI_NBA,
                Self::XAI_WNBA,
                Self::XAI_AMERICAN_FOOTBALL,
                Self::XAI_NCAA_FOOTBALL,
                Self::XAI_NFL,
                Self::XAI_BASEBALL,
                Self::XAI_MLB,
                Self::XAI_TENNIS,
                Self::XAI_CRICKET,
                Self::XAI_IPL,
                Self::XAI_MMA,
                Self::XAI_WWE,
                Self::XAI_BOXING,
                Self::XAI_GOLF,
                Self::XAI_RACING,
                Self::XAI_FORMULA1,
                Self::XAI_NASCAR,
                Self::XAI_ICE_HOCKEY,
                Self::XAI_NHL,
                Self::XAI_OLYMPICS,
                Self::XAI_RUGBY,
                Self::XAI_CYCLING,
                Self::XAI_SKIING,
                Self::XAI_SNOWBOARDING,
                Self::XAI_ESPORTS,
                Self::XAI_MOTO_GP,
                Self::XAI_PREMIER_LEAGUE,
                Self::XAI_SERIE_A,
            ]),
            Self::XAI_POLITICS => Some(&[Self::XAI_POLITICS]),
            Self::XAI_AI => Some(&[Self::XAI_AI]),
            Self::XAI_GAMING => Some(&[Self::XAI_GAMING]),
            Self::XAI_CRYPTOCURRENCY => Some(&[Self::XAI_CRYPTOCURRENCY]),
            Self::XAI_US_IRAN_WAR => Some(&[Self::XAI_US_IRAN_WAR]),
            Self::XAI_NEWS => Some(&[Self::XAI_NEWS, Self::XAI_NATURAL_DISASTERS]),
            Self::XAI_MOVIES_TV => Some(&[Self::XAI_MOVIES_TV, Self::XAI_STREAMING]),
            Self::XAI_MUSIC => Some(&[Self::XAI_MUSIC]),
            Self::XAI_DANCE => Some(&[Self::XAI_DANCE]),
            Self::XAI_CELEBRITY => Some(&[Self::XAI_CELEBRITY]),
            Self::XAI_ANIME => Some(&[Self::XAI_ANIME]),
            Self::XAI_SOCCER => Some(&[
                Self::XAI_SOCCER,
                Self::XAI_BUNDESLIGA,
                Self::XAI_LA_LIGA,
                Self::XAI_LIGUE1,
                Self::XAI_UEFA_CL,
                Self::XAI_UEFA_EL,
                Self::XAI_UEFA_ECL,
            ]),
            Self::XAI_BASKETBALL => Some(&[Self::XAI_BASKETBALL, Self::XAI_NBA, Self::XAI_WNBA]),
            Self::XAI_AMERICAN_FOOTBALL => Some(&[
                Self::XAI_AMERICAN_FOOTBALL,
                Self::XAI_NCAA_FOOTBALL,
                Self::XAI_NFL,
            ]),
            Self::XAI_BASEBALL => Some(&[Self::XAI_BASEBALL, Self::XAI_MLB]),
            Self::XAI_TENNIS => Some(&[Self::XAI_TENNIS]),
            Self::XAI_CRICKET => Some(&[Self::XAI_CRICKET, Self::XAI_IPL]),
            Self::XAI_MMA => Some(&[Self::XAI_MMA, Self::XAI_WWE]),
            Self::XAI_BOXING => Some(&[Self::XAI_BOXING]),
            Self::XAI_GOLF => Some(&[Self::XAI_GOLF]),
            Self::XAI_RACING => Some(&[Self::XAI_RACING, Self::XAI_FORMULA1, Self::XAI_NASCAR]),
            Self::XAI_ICE_HOCKEY => Some(&[Self::XAI_ICE_HOCKEY, Self::XAI_NHL]),
            Self::XAI_OLYMPICS => Some(&[Self::XAI_OLYMPICS]),
            Self::XAI_RUGBY => Some(&[Self::XAI_RUGBY]),
            Self::XAI_CYCLING => Some(&[Self::XAI_CYCLING]),
            Self::XAI_SKIING => Some(&[Self::XAI_SKIING, Self::XAI_SNOWBOARDING]),
            Self::XAI_HEALTH_FITNESS => Some(&[
                Self::XAI_HEALTH_FITNESS,
                Self::XAI_NUTRITION,
                Self::XAI_WORKOUTS,
            ]),
            Self::XAI_TRAVEL => Some(&[Self::XAI_TRAVEL, Self::XAI_AVIATION]),
            Self::XAI_FOOD => Some(&[
                Self::XAI_FOOD,
                Self::XAI_COOKING,
                Self::XAI_BAKING,
                Self::XAI_RESTAURANTS,
                Self::XAI_DRINKS,
            ]),
            Self::XAI_MEMES => Some(&[Self::XAI_MEMES]),
            Self::XAI_BEAUTY => Some(&[Self::XAI_BEAUTY]),
            Self::XAI_FASHION => Some(&[Self::XAI_FASHION]),
            Self::XAI_NATURE_OUTDOORS => Some(&[Self::XAI_NATURE_OUTDOORS]),
            Self::XAI_PETS => Some(&[Self::XAI_PETS, Self::XAI_CATS, Self::XAI_DOGS]),
            Self::XAI_HOME_GARDEN => Some(&[Self::XAI_HOME_GARDEN]),
            Self::XAI_ART => Some(&[Self::XAI_ART]),
            Self::XAI_RELIGION => Some(&[Self::XAI_RELIGION]),
            Self::XAI_SHOPPING => Some(&[Self::XAI_SHOPPING]),
            Self::XAI_EDUCATION => Some(&[Self::XAI_EDUCATION]),
            Self::XAI_CAREER => Some(&[Self::XAI_CAREER]),
            Self::XAI_CARS => Some(&[Self::XAI_CARS]),
            Self::XAI_MOTORCYCLES => Some(&[Self::XAI_MOTORCYCLES]),
            Self::XAI_RELATIONSHIPS => Some(&[Self::XAI_RELATIONSHIPS]),
            Self::XAI_FAMILY => Some(&[Self::XAI_FAMILY, Self::XAI_MARRIAGE, Self::XAI_PARENTING]),
            Self::XAI_PODCASTS => Some(&[Self::XAI_PODCASTS]),
            Self::XAI_POP => Some(&[Self::XAI_POP]),
            Self::XAI_K_POP => Some(&[Self::XAI_K_POP]),
            Self::XAI_COUNTRY_MUSIC => Some(&[Self::XAI_COUNTRY_MUSIC]),
            Self::XAI_ELECTRONIC => Some(&[Self::XAI_ELECTRONIC]),
            Self::XAI_J_POP => Some(&[Self::XAI_J_POP]),
            Self::XAI_ROCK => Some(&[Self::XAI_ROCK]),
            Self::XAI_PHOTOGRAPHY => Some(&[Self::XAI_PHOTOGRAPHY]),
            Self::XAI_DESIGN => Some(&[Self::XAI_DESIGN]),
            Self::XAI_SOFTWARE_DEVELOPMENT => Some(&[Self::XAI_SOFTWARE_DEVELOPMENT]),
            Self::XAI_ROBOTICS => Some(&[Self::XAI_ROBOTICS]),
            Self::XAI_SPACE => Some(&[Self::XAI_SPACE]),
            Self::XAI_BIOTECH => Some(&[Self::XAI_BIOTECH]),
            Self::XAI_ELECTRONICS => Some(&[Self::XAI_ELECTRONICS]),
            Self::XAI_STOCKS => Some(&[Self::XAI_STOCKS]),
            Self::XAI_ENTREPRENEURSHIP => Some(&[Self::XAI_ENTREPRENEURSHIP]),
            Self::XAI_REAL_ESTATE => Some(&[Self::XAI_REAL_ESTATE]),
            Self::XAI_PERSONAL_FINANCE => Some(&[Self::XAI_PERSONAL_FINANCE]),
            Self::XAI_CRIME => Some(&[Self::XAI_CRIME]),
            Self::XAI_DATING => Some(&[Self::XAI_DATING]),
            Self::XAI_ELECTIONS => Some(&[Self::XAI_ELECTIONS]),
            Self::XAI_ESPORTS => Some(&[Self::XAI_ESPORTS]),
            Self::XAI_HIP_HOP => Some(&[Self::XAI_HIP_HOP]),
            Self::XAI_JAZZ => Some(&[Self::XAI_JAZZ]),
            Self::XAI_CONCERTS => Some(&[Self::XAI_CONCERTS]),
            Self::XAI_MENTAL_HEALTH => Some(&[Self::XAI_MENTAL_HEALTH]),
            Self::XAI_MOTO_GP => Some(&[Self::XAI_MOTO_GP]),
            Self::XAI_PREMIER_LEAGUE => Some(&[Self::XAI_PREMIER_LEAGUE]),
            Self::XAI_SERIE_A => Some(&[Self::XAI_SERIE_A]),
            Self::XAI_CHRISTIANITY => Some(&[Self::XAI_CHRISTIANITY]),
            Self::XAI_BUDDHISM => Some(&[Self::XAI_BUDDHISM]),
            Self::XAI_HINDUISM => Some(&[Self::XAI_HINDUISM]),
            Self::XAI_ISLAM => Some(&[Self::XAI_ISLAM]),
            Self::XAI_JUDAISM => Some(&[Self::XAI_JUDAISM]),
            Self::XAI_DIGITAL_ART => Some(&[Self::XAI_DIGITAL_ART]),
            Self::XAI_SCIENCE => Some(&[Self::XAI_SCIENCE]),
            Self::XAI_TECHNOLOGY => Some(&[Self::XAI_TECHNOLOGY]),
            Self::XAI_BUSINESS_FINANCE_REAL => Some(&[Self::XAI_BUSINESS_FINANCE_REAL]),
            Self::XAI_SPORTS_REAL => Some(&[Self::XAI_SPORTS_REAL]),
            Self::XAI_ATHLETICS => Some(&[Self::XAI_ATHLETICS]),
            Self::XAI_STREAMING => Some(&[Self::XAI_STREAMING]),
            Self::XAI_STOCKS_ECONOMY => Some(&[Self::XAI_STOCKS_ECONOMY]),
            _ => None,
        }
    }

    pub fn expand(topic_ids: &HashSet<i64>) -> HashSet<i64> {
        let mut expanded = HashSet::new();
        for &topic_id in topic_ids {
            match Self::category_ids(topic_id) {
                Some(ids) => {
                    for &id in ids {
                        expanded.insert(id);
                    }
                }
                None => {
                    expanded.insert(topic_id);
                }
            }
        }
        expanded
    }

    pub fn all_production_topic_ids() -> HashSet<i64> {
        let all_client_topics: HashSet<i64> = [
            Self::SCIENCE_TECHNOLOGY,
            Self::ENTERTAINMENT,
            Self::BUSINESS_FINANCE,
            Self::SPORTS,
            Self::XAI_POLITICS,
            Self::XAI_AI,
            Self::XAI_GAMING,
            Self::XAI_CRYPTOCURRENCY,
            Self::XAI_US_IRAN_WAR,
            Self::XAI_NEWS,
            Self::XAI_MOVIES_TV,
            Self::XAI_MUSIC,
            Self::XAI_DANCE,
            Self::XAI_CELEBRITY,
            Self::XAI_ANIME,
            Self::XAI_SOCCER,
            Self::XAI_BASKETBALL,
            Self::XAI_AMERICAN_FOOTBALL,
            Self::XAI_BASEBALL,
            Self::XAI_TENNIS,
            Self::XAI_CRICKET,
            Self::XAI_MMA,
            Self::XAI_BOXING,
            Self::XAI_GOLF,
            Self::XAI_RACING,
            Self::XAI_ICE_HOCKEY,
            Self::XAI_OLYMPICS,
            Self::XAI_RUGBY,
            Self::XAI_CYCLING,
            Self::XAI_SKIING,
            Self::XAI_HEALTH_FITNESS,
            Self::XAI_TRAVEL,
            Self::XAI_FOOD,
            Self::XAI_MEMES,
            Self::XAI_BEAUTY,
            Self::XAI_FASHION,
            Self::XAI_NATURE_OUTDOORS,
            Self::XAI_PETS,
            Self::XAI_HOME_GARDEN,
            Self::XAI_ART,
            Self::XAI_RELIGION,
            Self::XAI_SHOPPING,
            Self::XAI_EDUCATION,
            Self::XAI_CAREER,
            Self::XAI_CARS,
            Self::XAI_MOTORCYCLES,
            Self::XAI_RELATIONSHIPS,
            Self::XAI_FAMILY,
            Self::XAI_PODCASTS,
            Self::XAI_POP,
            Self::XAI_K_POP,
            Self::XAI_COUNTRY_MUSIC,
            Self::XAI_ELECTRONIC,
            Self::XAI_J_POP,
            Self::XAI_ROCK,
            Self::XAI_PHOTOGRAPHY,
            Self::XAI_DESIGN,
            Self::XAI_SOFTWARE_DEVELOPMENT,
            Self::XAI_ROBOTICS,
            Self::XAI_SPACE,
            Self::XAI_BIOTECH,
            Self::XAI_ELECTRONICS,
            Self::XAI_STOCKS,
            Self::XAI_ENTREPRENEURSHIP,
            Self::XAI_REAL_ESTATE,
            Self::XAI_PERSONAL_FINANCE,
            Self::XAI_CRIME,
            Self::XAI_DATING,
            Self::XAI_ELECTIONS,
            Self::XAI_ESPORTS,
            Self::XAI_HIP_HOP,
            Self::XAI_JAZZ,
            Self::XAI_CONCERTS,
            Self::XAI_MENTAL_HEALTH,
            Self::XAI_MOTO_GP,
            Self::XAI_PREMIER_LEAGUE,
            Self::XAI_SERIE_A,
            Self::XAI_CHRISTIANITY,
            Self::XAI_BUDDHISM,
            Self::XAI_HINDUISM,
            Self::XAI_ISLAM,
            Self::XAI_JUDAISM,
            Self::XAI_DIGITAL_ART,
            Self::XAI_STOCKS_ECONOMY,
        ]
        .into();
        Self::expand(&all_client_topics)
    }

    pub fn resolve_first(topic_id: i64) -> i64 {
        topic_id
    }

    pub fn supertopic(topic_id: i64) -> i64 {
        match topic_id {
            Self::XAI_POP
            | Self::XAI_K_POP
            | Self::XAI_COUNTRY_MUSIC
            | Self::XAI_ELECTRONIC
            | Self::XAI_J_POP
            | Self::XAI_ROCK
            | Self::XAI_HIP_HOP
            | Self::XAI_JAZZ
            | Self::XAI_CONCERTS => Self::XAI_MUSIC,

            Self::XAI_DESIGN | Self::XAI_DIGITAL_ART | Self::XAI_PHOTOGRAPHY => Self::XAI_ART,

            Self::XAI_NATURAL_DISASTERS | Self::XAI_CRIME => Self::XAI_NEWS,

            Self::XAI_COOKING | Self::XAI_BAKING | Self::XAI_RESTAURANTS | Self::XAI_DRINKS => {
                Self::XAI_FOOD
            }

            Self::XAI_AVIATION => Self::XAI_TRAVEL,

            Self::XAI_CATS | Self::XAI_DOGS => Self::XAI_PETS,

            Self::XAI_CHRISTIANITY
            | Self::XAI_BUDDHISM
            | Self::XAI_HINDUISM
            | Self::XAI_ISLAM
            | Self::XAI_JUDAISM => Self::XAI_RELIGION,

            Self::XAI_NUTRITION | Self::XAI_WORKOUTS | Self::XAI_MENTAL_HEALTH => {
                Self::XAI_HEALTH_FITNESS
            }

            Self::XAI_DATING | Self::XAI_MARRIAGE => Self::XAI_RELATIONSHIPS,

            Self::XAI_PARENTING => Self::XAI_FAMILY,

            Self::XAI_ELECTIONS => Self::XAI_POLITICS,

            Self::XAI_CRYPTOCURRENCY
            | Self::XAI_STOCKS
            | Self::XAI_ENTREPRENEURSHIP
            | Self::XAI_REAL_ESTATE
            | Self::XAI_PERSONAL_FINANCE => Self::XAI_BUSINESS_FINANCE_REAL,

            Self::XAI_AI
            | Self::XAI_SOFTWARE_DEVELOPMENT
            | Self::XAI_ROBOTICS
            | Self::XAI_ELECTRONICS => Self::XAI_TECHNOLOGY,

            Self::XAI_BIOTECH | Self::XAI_SPACE => Self::XAI_SCIENCE,

            Self::XAI_BUNDESLIGA
            | Self::XAI_LA_LIGA
            | Self::XAI_LIGUE1
            | Self::XAI_UEFA_CL
            | Self::XAI_UEFA_EL
            | Self::XAI_UEFA_ECL
            | Self::XAI_PREMIER_LEAGUE
            | Self::XAI_SERIE_A => Self::XAI_SOCCER,

            Self::XAI_NBA | Self::XAI_WNBA => Self::XAI_BASKETBALL,

            Self::XAI_NCAA_FOOTBALL | Self::XAI_NFL => Self::XAI_AMERICAN_FOOTBALL,

            Self::XAI_MLB => Self::XAI_BASEBALL,

            Self::XAI_IPL => Self::XAI_CRICKET,

            Self::XAI_NHL => Self::XAI_ICE_HOCKEY,

            Self::XAI_FORMULA1 | Self::XAI_NASCAR | Self::XAI_MOTO_GP => Self::XAI_RACING,

            Self::XAI_ESPORTS => Self::XAI_GAMING,

            Self::XAI_MOTORCYCLES => Self::XAI_CARS,

            Self::XAI_STREAMING => Self::XAI_MOVIES_TV,

            Self::XAI_AMERICAN_FOOTBALL
            | Self::XAI_ATHLETICS
            | Self::XAI_BASEBALL
            | Self::XAI_BASKETBALL
            | Self::XAI_BOXING
            | Self::XAI_CRICKET
            | Self::XAI_CYCLING
            | Self::XAI_GOLF
            | Self::XAI_ICE_HOCKEY
            | Self::XAI_MMA
            | Self::XAI_OLYMPICS
            | Self::XAI_RACING
            | Self::XAI_RUGBY
            | Self::XAI_SKIING
            | Self::XAI_SNOWBOARDING
            | Self::XAI_SOCCER
            | Self::XAI_TENNIS
            | Self::XAI_WWE => Self::XAI_SPORTS_REAL,

            _ => topic_id,
        }
    }

    pub fn expand_supertopic(topic_id: i64) -> HashSet<i64> {
        let st = Self::supertopic(topic_id);
        match Self::category_ids(st) {
            Some(ids) => ids.iter().copied().collect(),
            None => [st].into(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TopicFilteringOverrideMap {
    overrides: HashMap<i64, TopicFilteringExperiment>,
}

impl TopicFilteringOverrideMap {
    pub fn parse(raw: &str) -> Self {
        let mut overrides = HashMap::new();
        if raw.is_empty() {
            return Self { overrides };
        }
        for entry in raw.split(',') {
            let entry = entry.trim();
            if entry.is_empty() {
                continue;
            }
            if let Some((topic_str, experiment_str)) = entry.split_once('=') {
                match topic_str.trim().parse::<i64>() {
                    Ok(topic_id) => {
                        let experiment = TopicFilteringExperiment::parse(experiment_str.trim());
                        overrides.insert(topic_id, experiment);
                    }
                    Err(_) => {
                        warn!(
                            "TopicFilteringOverrides: invalid topic_id '{}' in entry '{}'",
                            topic_str, entry
                        );
                    }
                }
            } else {
                warn!(
                    "TopicFilteringOverrides: malformed entry '{}', expected 'topic_id=ExperimentId'",
                    entry
                );
            }
        }
        Self { overrides }
    }

    pub fn resolve(
        &self,
        query_topic_ids: &[i64],
        default: TopicFilteringExperiment,
    ) -> TopicFilteringExperiment {
        if self.overrides.is_empty() {
            return default;
        }
        for &tid in query_topic_ids {
            if let Some(&exp) = self.overrides.get(&tid) {
                return exp;
            }
        }
        default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_news_includes_news_and_natural_disasters() {
        let ids: HashSet<i64> = [TopicIdExpansion::XAI_NEWS].into();
        let expanded = TopicIdExpansion::expand(&ids);
        assert!(expanded.contains(&TopicIdExpansion::XAI_NEWS));
        assert!(expanded.contains(&1925952795854745601));
        assert_eq!(expanded.len(), 2);
    }

    #[test]
    fn test_expand_entertainment_group() {
        let ids: HashSet<i64> = [TopicIdExpansion::ENTERTAINMENT].into();
        let expanded = TopicIdExpansion::expand(&ids);
        assert!(expanded.contains(&TopicIdExpansion::XAI_MOVIES_TV));
        assert!(expanded.contains(&TopicIdExpansion::XAI_MUSIC));
        assert!(expanded.contains(&TopicIdExpansion::XAI_GAMING));
        assert!(expanded.contains(&TopicIdExpansion::XAI_ANIME));
        assert!(expanded.contains(&1925950455806369792));
        assert!(expanded.contains(&TopicIdExpansion::XAI_DANCE));
        assert!(expanded.contains(&TopicIdExpansion::XAI_CELEBRITY));
    }

    #[test]
    fn test_expand_science_technology_group() {
        let ids: HashSet<i64> = [TopicIdExpansion::SCIENCE_TECHNOLOGY].into();
        let expanded = TopicIdExpansion::expand(&ids);
        assert!(expanded.contains(&TopicIdExpansion::XAI_SCIENCE));
        assert!(expanded.contains(&TopicIdExpansion::XAI_TECHNOLOGY));
        assert!(expanded.contains(&TopicIdExpansion::XAI_AI));
        assert!(expanded.contains(&1925953040130953216));
        assert!(expanded.contains(&1925953169307189248));
        assert!(expanded.contains(&1925953141452718081));
        assert!(expanded.contains(&1925953107038519296));
        assert!(expanded.contains(&1925953073983139840));
    }

    #[test]
    fn test_expand_business_finance_group() {
        let ids: HashSet<i64> = [TopicIdExpansion::BUSINESS_FINANCE].into();
        let expanded = TopicIdExpansion::expand(&ids);
        assert!(expanded.contains(&TopicIdExpansion::XAI_BUSINESS_FINANCE_REAL));
        assert!(expanded.contains(&TopicIdExpansion::XAI_CRYPTOCURRENCY));
        assert!(expanded.contains(&1925952876284727296));
        assert!(expanded.contains(&1925952916411564032));
        assert!(expanded.contains(&1925952947197784064));
        assert!(expanded.contains(&1925952988486447104));
    }

    #[test]
    fn test_expand_sports_group() {
        let ids: HashSet<i64> = [TopicIdExpansion::SPORTS].into();
        let expanded = TopicIdExpansion::expand(&ids);
        assert!(expanded.contains(&TopicIdExpansion::XAI_SPORTS_REAL));
        assert!(expanded.contains(&1925950838414942210));
        assert!(expanded.contains(&1925950498420469760));
        assert!(expanded.contains(&1925950530842464256));
        assert!(expanded.contains(&1925950554133446656));
        assert!(expanded.contains(&1925950576740675584));
        assert!(expanded.contains(&1925950625835008001));
    }

    #[test]
    fn test_expand_unknown_topic_passed_through() {
        let ids: HashSet<i64> = [12345].into();
        let expanded = TopicIdExpansion::expand(&ids);
        assert_eq!(expanded, [12345].into());
    }

    #[test]
    fn test_filter_enables_for_any_topic_request() {
        let query_regular = ScoredPostsQuery {
            topic_ids: vec![TopicIdExpansion::XAI_SPORTS_REAL],
            ..Default::default()
        };
        assert!(TopicIdsFilter.enable(&query_regular));

        let query_bulk = ScoredPostsQuery {
            topic_ids: vec![1, 2, 3, 4, 5, 6, 7],
            ..Default::default()
        };
        assert!(TopicIdsFilter.enable(&query_bulk));

        let query_excluded = ScoredPostsQuery {
            excluded_topic_ids: vec![TopicIdExpansion::XAI_SPORTS_REAL],
            ..Default::default()
        };
        assert!(TopicIdsFilter.enable(&query_excluded));

        let query_none = ScoredPostsQuery {
            topic_ids: vec![],
            ..Default::default()
        };
        assert!(!TopicIdsFilter.enable(&query_none));
    }

    #[test]
    fn test_inclusion_mode_keeps_only_matching() {
        let query = ScoredPostsQuery {
            topic_ids: vec![TopicIdExpansion::XAI_SPORTS_REAL],
            ..Default::default()
        };

        let candidates = vec![
            PostCandidate {
                tweet_id: 1,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_SPORTS_REAL]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 2,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_AI]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 3,
                filtered_topic_ids: None,
                ..Default::default()
            },
        ];

        let result = TopicIdsFilter.filter(&query, candidates);
        assert_eq!(result.kept.len(), 1);
        assert_eq!(result.kept[0].tweet_id, 1);
        assert_eq!(result.removed.len(), 2);
    }

    #[test]
    fn test_bulk_filter_removes_only_excluded_topics() {
        let query = ScoredPostsQuery {
            topic_ids: vec![
                TopicIdExpansion::XAI_NEWS,
                TopicIdExpansion::BUSINESS_FINANCE,
                TopicIdExpansion::SCIENCE_TECHNOLOGY,
                TopicIdExpansion::XAI_MOVIES_TV,
                TopicIdExpansion::XAI_AI,
                TopicIdExpansion::XAI_GAMING,
                TopicIdExpansion::XAI_CRYPTOCURRENCY,
            ],
            ..Default::default()
        };

        let candidates = vec![
            PostCandidate {
                tweet_id: 1,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_NEWS]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 2,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_SPORTS_REAL]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 3,
                filtered_topic_ids: None,
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 4,
                filtered_topic_ids: Some(vec![]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 5,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_US_IRAN_WAR]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 6,
                filtered_topic_ids: Some(vec![
                    TopicIdExpansion::XAI_NEWS,
                    TopicIdExpansion::XAI_SPORTS_REAL,
                ]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 7,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_CRYPTOCURRENCY]),
                ..Default::default()
            },
        ];

        let result = TopicIdsFilter.filter(&query, candidates);
        let kept_ids: Vec<u64> = result.kept.iter().map(|c| c.tweet_id).collect();
        let removed_ids: Vec<u64> = result.removed.iter().map(|c| c.tweet_id).collect();
        assert_eq!(kept_ids, vec![1, 3, 4, 6, 7]);
        assert_eq!(removed_ids, vec![2, 5]);
    }

    #[test]
    fn test_expand_us_iran_war_maps_to_self() {
        let ids: HashSet<i64> = [TopicIdExpansion::XAI_US_IRAN_WAR].into();
        let expanded = TopicIdExpansion::expand(&ids);
        assert_eq!(expanded.len(), 1);
        assert!(expanded.contains(&TopicIdExpansion::XAI_US_IRAN_WAR));
    }

    #[test]
    fn test_resolve_first_us_iran_war() {
        assert_eq!(
            TopicIdExpansion::resolve_first(TopicIdExpansion::XAI_US_IRAN_WAR),
            TopicIdExpansion::XAI_US_IRAN_WAR,
        );
    }

    #[test]
    fn test_resolve_first_is_identity() {
        assert_eq!(
            TopicIdExpansion::resolve_first(TopicIdExpansion::SCIENCE_TECHNOLOGY),
            TopicIdExpansion::SCIENCE_TECHNOLOGY,
        );
        assert_eq!(
            TopicIdExpansion::resolve_first(TopicIdExpansion::ENTERTAINMENT),
            TopicIdExpansion::ENTERTAINMENT,
        );
        assert_eq!(
            TopicIdExpansion::resolve_first(TopicIdExpansion::XAI_NEWS),
            TopicIdExpansion::XAI_NEWS,
        );
        assert_eq!(
            TopicIdExpansion::resolve_first(TopicIdExpansion::XAI_AI),
            TopicIdExpansion::XAI_AI,
        );
        assert_eq!(
            TopicIdExpansion::resolve_first(TopicIdExpansion::XAI_POLITICS),
            TopicIdExpansion::XAI_POLITICS,
        );
        assert_eq!(TopicIdExpansion::resolve_first(12345), 12345,);
    }

    #[test]
    fn test_filter_keeps_us_iran_war_candidates() {
        let query = ScoredPostsQuery {
            topic_ids: vec![TopicIdExpansion::XAI_US_IRAN_WAR],
            ..Default::default()
        };

        let candidates = vec![
            PostCandidate {
                tweet_id: 10,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_US_IRAN_WAR]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 11,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_SPORTS_REAL]),
                ..Default::default()
            },
        ];

        let result = TopicIdsFilter.filter(&query, candidates);
        assert_eq!(result.kept.len(), 1);
        assert_eq!(result.kept[0].tweet_id, 10);
    }

    #[test]
    fn test_override_map_parse_empty() {
        let map = TopicFilteringOverrideMap::parse("");
        assert_eq!(
            map.resolve(
                &[2000000000000000001],
                TopicFilteringExperiment::PostBased90Pct
            ),
            TopicFilteringExperiment::PostBased90Pct,
        );
    }

    #[test]
    fn test_override_map_parse_single() {
        let map = TopicFilteringOverrideMap::parse("2000000000000000001=Unfiltered");
        assert_eq!(
            map.resolve(
                &[2000000000000000001],
                TopicFilteringExperiment::PostBased90Pct
            ),
            TopicFilteringExperiment::Unfiltered,
        );
        assert_eq!(
            map.resolve(&[12345], TopicFilteringExperiment::PostBased90Pct),
            TopicFilteringExperiment::PostBased90Pct,
        );
    }

    #[test]
    fn test_override_map_parse_multiple() {
        let map = TopicFilteringOverrideMap::parse(
            "2000000000000000001=Unfiltered, 1925949452197478400=CuratedV0",
        );
        assert_eq!(
            map.resolve(
                &[2000000000000000001],
                TopicFilteringExperiment::PostBased90Pct
            ),
            TopicFilteringExperiment::Unfiltered,
        );
        assert_eq!(
            map.resolve(
                &[TopicIdExpansion::XAI_SPORTS_REAL],
                TopicFilteringExperiment::PostBased90Pct
            ),
            TopicFilteringExperiment::CuratedV0,
        );
    }

    #[test]
    fn test_override_map_skips_malformed_entries() {
        let map = TopicFilteringOverrideMap::parse(
            "bad,notanumber=Unfiltered,2000000000000000001=PostBased90Pct",
        );
        assert_eq!(
            map.resolve(&[2000000000000000001], TopicFilteringExperiment::CuratedV0),
            TopicFilteringExperiment::PostBased90Pct,
        );
    }

    #[test]
    fn test_override_map_unknown_experiment_falls_back_to_unfiltered() {
        let map = TopicFilteringOverrideMap::parse("2000000000000000001=SomeNewExperiment");
        assert_eq!(
            map.resolve(
                &[2000000000000000001],
                TopicFilteringExperiment::PostBased90Pct
            ),
            TopicFilteringExperiment::Unfiltered,
        );
    }

    #[test]
    fn test_resolve_no_overrides_returns_default() {
        let map = TopicFilteringOverrideMap::parse("");
        let topics = vec![TopicIdExpansion::XAI_SPORTS_REAL];
        assert_eq!(
            map.resolve(&topics, TopicFilteringExperiment::PostBased90Pct),
            TopicFilteringExperiment::PostBased90Pct,
        );
    }

    #[test]
    fn test_resolve_override_wins_over_default() {
        let map = TopicFilteringOverrideMap::parse("2000000000000000001=Unfiltered");
        let topics = vec![TopicIdExpansion::XAI_SPORTS_REAL, 2000000000000000001];
        assert_eq!(
            map.resolve(&topics, TopicFilteringExperiment::PostBased90Pct),
            TopicFilteringExperiment::Unfiltered,
        );
    }

    #[test]
    fn test_resolve_first_override_wins_tiebreaker() {
        let map = TopicFilteringOverrideMap::parse(
            "2000000000000000001=Unfiltered,1925949452197478400=CuratedV0",
        );
        let topics = vec![2000000000000000001, TopicIdExpansion::XAI_SPORTS_REAL];
        assert_eq!(
            map.resolve(&topics, TopicFilteringExperiment::PostBased90Pct),
            TopicFilteringExperiment::Unfiltered,
        );
        let topics = vec![TopicIdExpansion::XAI_SPORTS_REAL, 2000000000000000001];
        assert_eq!(
            map.resolve(&topics, TopicFilteringExperiment::PostBased90Pct),
            TopicFilteringExperiment::CuratedV0,
        );
    }

    #[test]
    fn test_resolve_no_matching_override_returns_default() {
        let map = TopicFilteringOverrideMap::parse("2000000000000000001=Unfiltered");
        let topics = vec![TopicIdExpansion::XAI_SPORTS_REAL];
        assert_eq!(
            map.resolve(&topics, TopicFilteringExperiment::PostBased90Pct),
            TopicFilteringExperiment::PostBased90Pct,
        );
    }

    #[test]
    fn test_excluded_topics_removes_matching_posts() {
        let query = ScoredPostsQuery {
            excluded_topic_ids: vec![TopicIdExpansion::XAI_SPORTS_REAL],
            ..Default::default()
        };

        let candidates = vec![
            PostCandidate {
                tweet_id: 1,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_SPORTS_REAL]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 2,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_AI]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 3,
                filtered_topic_ids: None,
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 4,
                filtered_topic_ids: Some(vec![]),
                ..Default::default()
            },
        ];

        let result = TopicIdsFilter.filter(&query, candidates);
        let kept_ids: Vec<u64> = result.kept.iter().map(|c| c.tweet_id).collect();
        let removed_ids: Vec<u64> = result.removed.iter().map(|c| c.tweet_id).collect();
        assert_eq!(kept_ids, vec![2]);
        assert_eq!(removed_ids, vec![1, 3, 4]);
    }

    #[test]
    fn test_excluded_topics_removes_any_match() {
        let query = ScoredPostsQuery {
            excluded_topic_ids: vec![TopicIdExpansion::XAI_SPORTS_REAL],
            ..Default::default()
        };

        let candidates = vec![PostCandidate {
            tweet_id: 1,
            filtered_topic_ids: Some(vec![
                TopicIdExpansion::XAI_SPORTS_REAL,
                TopicIdExpansion::XAI_POLITICS,
            ]),
            ..Default::default()
        }];

        let result = TopicIdsFilter.filter(&query, candidates);
        assert_eq!(result.kept.len(), 0);
        assert_eq!(result.removed.len(), 1);
    }

    #[test]
    fn test_excluded_entertainment_group() {
        let query = ScoredPostsQuery {
            excluded_topic_ids: vec![TopicIdExpansion::ENTERTAINMENT],
            ..Default::default()
        };

        let candidates = vec![
            PostCandidate {
                tweet_id: 1,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_GAMING]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 2,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_ANIME]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 3,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_POLITICS]),
                ..Default::default()
            },
        ];

        let result = TopicIdsFilter.filter(&query, candidates);
        let kept_ids: Vec<u64> = result.kept.iter().map(|c| c.tweet_id).collect();
        assert_eq!(kept_ids, vec![3]);
        assert_eq!(result.removed.len(), 2);
    }

    #[test]
    fn test_excluded_topics_not_a_topic_request() {
        let query = ScoredPostsQuery {
            excluded_topic_ids: vec![TopicIdExpansion::XAI_SPORTS_REAL],
            ..Default::default()
        };
        assert!(!query.is_topic_request());
        assert!(query.has_excluded_topics());
    }

    #[test]
    fn test_mixed_inclusion_and_exclusion() {
        let query = ScoredPostsQuery {
            topic_ids: vec![TopicIdExpansion::XAI_AI],
            excluded_topic_ids: vec![TopicIdExpansion::XAI_US_IRAN_WAR],
            ..Default::default()
        };

        let candidates = vec![
            PostCandidate {
                tweet_id: 1,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_AI]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 2,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_US_IRAN_WAR]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 3,
                filtered_topic_ids: Some(vec![
                    TopicIdExpansion::XAI_AI,
                    TopicIdExpansion::XAI_US_IRAN_WAR,
                ]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 4,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_SPORTS_REAL]),
                ..Default::default()
            },
        ];

        let result = TopicIdsFilter.filter(&query, candidates);
        let kept_ids: Vec<u64> = result.kept.iter().map(|c| c.tweet_id).collect();
        assert_eq!(kept_ids, vec![1]);
        assert_eq!(result.removed.len(), 3);
    }

    #[test]
    fn test_multiple_excluded_topics() {
        let query = ScoredPostsQuery {
            excluded_topic_ids: vec![
                TopicIdExpansion::XAI_SPORTS_REAL,
                TopicIdExpansion::XAI_US_IRAN_WAR,
            ],
            ..Default::default()
        };

        let candidates = vec![
            PostCandidate {
                tweet_id: 1,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_SPORTS_REAL]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 2,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_US_IRAN_WAR]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 3,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_AI]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 4,
                filtered_topic_ids: None,
                ..Default::default()
            },
        ];

        let result = TopicIdsFilter.filter(&query, candidates);
        let kept_ids: Vec<u64> = result.kept.iter().map(|c| c.tweet_id).collect();
        assert_eq!(kept_ids, vec![3]);
        assert_eq!(result.removed.len(), 3);
    }

    #[test]
    fn test_excluded_news_removes_news_and_natural_disasters() {
        let query = ScoredPostsQuery {
            excluded_topic_ids: vec![TopicIdExpansion::XAI_NEWS],
            ..Default::default()
        };

        let candidates = vec![
            PostCandidate {
                tweet_id: 1,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_NEWS]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 2,
                filtered_topic_ids: Some(vec![1925952795854745601]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 3,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_SPORTS_REAL]),
                ..Default::default()
            },
        ];

        let result = TopicIdsFilter.filter(&query, candidates);
        let kept_ids: Vec<u64> = result.kept.iter().map(|c| c.tweet_id).collect();
        assert_eq!(kept_ids, vec![3]);
        assert_eq!(result.removed.len(), 2);
    }

    #[test]
    fn test_expand_granular_soccer() {
        let ids: HashSet<i64> = [1925950498420469760].into();
        let expanded = TopicIdExpansion::expand(&ids);
        assert!(expanded.contains(&1925950498420469760));
        assert!(expanded.contains(&1925951257107206144));
        assert!(expanded.contains(&1925951079100928001));
        assert!(expanded.contains(&1925951334777233408));
        assert_eq!(expanded.len(), 7);
    }

    #[test]
    fn test_supertopic_music_children() {
        assert_eq!(
            TopicIdExpansion::supertopic(TopicIdExpansion::XAI_POP),
            TopicIdExpansion::XAI_MUSIC
        );
        assert_eq!(
            TopicIdExpansion::supertopic(TopicIdExpansion::XAI_K_POP),
            TopicIdExpansion::XAI_MUSIC
        );
        assert_eq!(
            TopicIdExpansion::supertopic(TopicIdExpansion::XAI_ROCK),
            TopicIdExpansion::XAI_MUSIC
        );
        assert_eq!(
            TopicIdExpansion::supertopic(TopicIdExpansion::XAI_HIP_HOP),
            TopicIdExpansion::XAI_MUSIC
        );
        assert_eq!(
            TopicIdExpansion::supertopic(TopicIdExpansion::XAI_JAZZ),
            TopicIdExpansion::XAI_MUSIC
        );
        assert_eq!(
            TopicIdExpansion::supertopic(TopicIdExpansion::XAI_CONCERTS),
            TopicIdExpansion::XAI_MUSIC
        );
    }

    #[test]
    fn test_supertopic_sports_hierarchy() {
        assert_eq!(
            TopicIdExpansion::supertopic(TopicIdExpansion::XAI_SOCCER),
            TopicIdExpansion::XAI_SPORTS_REAL
        );
        assert_eq!(
            TopicIdExpansion::supertopic(TopicIdExpansion::XAI_BUNDESLIGA),
            TopicIdExpansion::XAI_SOCCER
        );
        assert_eq!(
            TopicIdExpansion::supertopic(TopicIdExpansion::XAI_NFL),
            TopicIdExpansion::XAI_AMERICAN_FOOTBALL
        );
        assert_eq!(
            TopicIdExpansion::supertopic(TopicIdExpansion::XAI_IPL),
            TopicIdExpansion::XAI_CRICKET
        );
    }

    #[test]
    fn test_supertopic_self_identity() {
        assert_eq!(
            TopicIdExpansion::supertopic(TopicIdExpansion::XAI_MUSIC),
            TopicIdExpansion::XAI_MUSIC
        );
        assert_eq!(
            TopicIdExpansion::supertopic(TopicIdExpansion::XAI_POLITICS),
            TopicIdExpansion::XAI_POLITICS
        );
        assert_eq!(
            TopicIdExpansion::supertopic(TopicIdExpansion::XAI_MEMES),
            TopicIdExpansion::XAI_MEMES
        );
        assert_eq!(
            TopicIdExpansion::supertopic(TopicIdExpansion::XAI_NEWS),
            TopicIdExpansion::XAI_NEWS
        );
        assert_eq!(TopicIdExpansion::supertopic(99999), 99999);
    }

    #[test]
    fn test_expand_supertopic() {
        let expanded = TopicIdExpansion::expand_supertopic(TopicIdExpansion::XAI_POP);
        assert!(expanded.contains(&TopicIdExpansion::XAI_MUSIC));

        let expanded = TopicIdExpansion::expand_supertopic(TopicIdExpansion::XAI_BUNDESLIGA);
        assert!(expanded.contains(&TopicIdExpansion::XAI_SOCCER));
        assert!(expanded.contains(&TopicIdExpansion::XAI_BUNDESLIGA));

        let expanded = TopicIdExpansion::expand_supertopic(TopicIdExpansion::XAI_MUSIC);
        assert!(expanded.contains(&TopicIdExpansion::XAI_MUSIC));
        assert_eq!(expanded.len(), 1);
    }

    #[test]
    fn test_inclusion_pop_music_supertopic_fallback() {
        let query = ScoredPostsQuery {
            topic_ids: vec![TopicIdExpansion::XAI_POP],
            ..Default::default()
        };

        let candidates = vec![
            PostCandidate {
                tweet_id: 1,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_POP]),
                unfiltered_topic_ids: Some(vec![
                    TopicIdExpansion::XAI_MUSIC,
                    TopicIdExpansion::XAI_POP,
                ]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 2,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_MUSIC]),
                unfiltered_topic_ids: Some(vec![
                    TopicIdExpansion::XAI_MUSIC,
                    TopicIdExpansion::XAI_POP,
                ]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 3,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_MUSIC]),
                unfiltered_topic_ids: Some(vec![TopicIdExpansion::XAI_MUSIC]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 4,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_AI]),
                unfiltered_topic_ids: Some(vec![TopicIdExpansion::XAI_AI]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 5,
                filtered_topic_ids: None,
                unfiltered_topic_ids: None,
                ..Default::default()
            },
        ];

        let result = TopicIdsFilter.filter(&query, candidates);
        let kept_ids: Vec<u64> = result.kept.iter().map(|c| c.tweet_id).collect();
        assert_eq!(kept_ids, vec![1, 2]);
        assert_eq!(result.removed.len(), 3);
    }

    #[test]
    fn test_inclusion_bundesliga_supertopic_fallback() {
        let query = ScoredPostsQuery {
            topic_ids: vec![TopicIdExpansion::XAI_BUNDESLIGA],
            ..Default::default()
        };

        let candidates = vec![
            PostCandidate {
                tweet_id: 1,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_SOCCER]),
                unfiltered_topic_ids: Some(vec![
                    TopicIdExpansion::XAI_SOCCER,
                    TopicIdExpansion::XAI_BUNDESLIGA,
                ]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 2,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_BUNDESLIGA]),
                unfiltered_topic_ids: Some(vec![TopicIdExpansion::XAI_BUNDESLIGA]),
                ..Default::default()
            },
        ];

        let result = TopicIdsFilter.filter(&query, candidates);
        let kept_ids: Vec<u64> = result.kept.iter().map(|c| c.tweet_id).collect();
        assert_eq!(kept_ids, vec![1, 2]);
    }

    #[test]
    fn test_inclusion_self_supertopic_no_regression() {
        let query = ScoredPostsQuery {
            topic_ids: vec![TopicIdExpansion::XAI_MUSIC],
            ..Default::default()
        };

        let candidates = vec![
            PostCandidate {
                tweet_id: 1,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_MUSIC]),
                unfiltered_topic_ids: Some(vec![TopicIdExpansion::XAI_MUSIC]),
                ..Default::default()
            },
            PostCandidate {
                tweet_id: 2,
                filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_AI]),
                unfiltered_topic_ids: Some(vec![TopicIdExpansion::XAI_AI]),
                ..Default::default()
            },
        ];

        let result = TopicIdsFilter.filter(&query, candidates);
        assert_eq!(result.kept.len(), 1);
        assert_eq!(result.kept[0].tweet_id, 1);
    }

    #[test]
    fn test_inclusion_no_unfiltered_topics_path2_fails() {
        let query = ScoredPostsQuery {
            topic_ids: vec![TopicIdExpansion::XAI_POP],
            ..Default::default()
        };

        let candidates = vec![PostCandidate {
            tweet_id: 1,
            filtered_topic_ids: Some(vec![TopicIdExpansion::XAI_MUSIC]),
            unfiltered_topic_ids: None,
            ..Default::default()
        }];

        let result = TopicIdsFilter.filter(&query, candidates);
        assert_eq!(result.kept.len(), 0);
    }
}
