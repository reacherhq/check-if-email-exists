# Auto-héberger Reacher, l’API open source de vérification d’adresses e-mail

Reacher vérifie des adresses e-mail sans envoyer de message. Son serveur HTTP est écrit en Rust et le code source est disponible dans le dépôt [reacherhq/check-if-email-exists](https://github.com/reacherhq/check-if-email-exists).

Pour essayer Reacher sur votre machine, installez Docker et vérifiez que votre hébergeur autorise les connexions SMTP sortantes sur le port 25. Lancez ensuite le serveur dans un terminal.

```sh
docker run --rm -p 127.0.0.1:8080:8080 reacherhq/backend:latest
```

Dans un autre terminal, remplacez l’adresse d’exemple par une adresse que vous contrôlez, puis envoyez une requête à l’API locale.

```sh
curl --json '{"to_email":"test@example.org"}' http://127.0.0.1:8080/v1/check_email
```

Le champ `is_reachable` résume le résultat. Un serveur distant qui accepte toutes les adresses de son domaine ou qui ne répond pas au test peut empêcher de conclure pour une boîte précise. Consultez le [guide des résultats](../getting-started/is-reachable.md) pour interpréter la réponse et le [guide de configuration](reacher-configuration-v0.10.md) pour adapter le serveur à votre installation.
