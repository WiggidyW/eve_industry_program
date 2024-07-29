SELECT
    products.portion,
    products.probability,
    blueprints.duration,
    blueprints.rigs_skills_structures,
    products.installation_minerals,
    blueprints.minerals
FROM
    blueprints
INNER JOIN
    products
ON
    products.id = blueprints.products
WHERE
    (products.type_id, blueprints.type_id, blueprints.kind) = (?, ?, ?);
