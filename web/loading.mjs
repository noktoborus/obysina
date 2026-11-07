// The page of loading of `index.html`: trunk calls these while it fetches
// the module of the program and starts it. The bar shows the bytes fetched
// of the bytes the server announced; a server that announces none, or the
// compressed size, gets a bar that sweeps instead.

const russian = (navigator.language || "").toLowerCase().startsWith("ru");
const words = russian
  ? { loading: "Загрузка", starting: "Запуск…", failed: "Не удалось запустить: " }
  : { loading: "Loading", starting: "Starting…", failed: "The program did not start: " };

const page = () => ({
  loading: document.getElementById("loading"),
  track: document.getElementById("track"),
  bar: document.getElementById("bar"),
  state: document.getElementById("state"),
});

/// Megabytes, with one digit after the point.
const megabytes = (bytes) => (bytes / 1048576).toFixed(1);

export default function () {
  return {
    onStart: () => {
      page().state.textContent = words.loading + "…";
    },
    onProgress: ({ current, total }) => {
      const { loading, track, bar, state } = page();
      if (!total || current > total) {
        track.classList.add("unknown");
        loading.removeAttribute("aria-valuenow");
        state.textContent = `${words.loading} ${megabytes(current)} MB`;
        return;
      }
      const share = Math.round((current / total) * 100);
      track.classList.remove("unknown");
      bar.style.width = share + "%";
      loading.setAttribute("aria-valuenow", share);
      state.textContent = `${words.loading} ${megabytes(current)} / ${megabytes(total)} MB`;
    },
    onComplete: () => {
      const { track, bar, state } = page();
      track.classList.remove("unknown");
      bar.style.width = "100%";
      state.textContent = words.starting;
    },
    onSuccess: () => {
      const { loading } = page();
      loading.classList.add("gone");
      setTimeout(() => loading.remove(), 400);
    },
    onFailure: (error) => {
      const { loading, state } = page();
      loading.classList.add("failed");
      state.textContent = words.failed + error;
    },
  };
}
