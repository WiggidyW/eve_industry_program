SELECT
    type_id,
    time_efficiency,
    material_efficiency,
    cost_efficiency,
    probability_multiplier,
    high_sec_multiplier,
    low_sec_multiplier,
    zero_sec_multiplier
FROM
    efficiencies
WHERE
    kind = ?
    AND
    type_id IN (
        SELECT
            type_id
        FROM
            rigs_skills_structures
        WHERE
            id = ?
    );
