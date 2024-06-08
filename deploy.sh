source_dir="$PWD"

if [ ! -d "gh-pages" ]; then
  origin=$(git remote get-url origin)
  git clone --depth=1 --branch "gh-pages" "$origin" "gh-pages"
  code=$?
  if [ $code != 0 ]; then
    mkdir "gh-pages"
    cd "gh-pages"
    git init
    git remote add origin "$origin"
    git checkout --orphan "gh-pages"
    git commit --allow-empty --message "create gh-pages"
    git push -u origin "gh-pages"
    cd "$source_dir"
  fi
fi

wasm-pack build --target web

cd "gh-pages"
git pull
rm -r *
cp ../pkg/*.js ../pkg/*.wasm ../web/index.html .
git add .
git commit -m "gh-pages"
git push

repo=$(git remote get-url origin | sed -r 's/git@github.com:(.*).git/\1/')

if [ -z ${repo+x} ]; then
  repo=$(git remote get-url origin | sed -r 's/https:\/\/github.com\/(.*).git/\1/')
fi

if [ -z ${repo+x} ]; then
  echo "unrecognized github origin: $origin"
  exit 1
fi

echo "https://github.com/$repo/commits/gh-pages"
