CREATE TABLE message_templates (
    key         VARCHAR(100) PRIMARY KEY,
    body        TEXT NOT NULL,
    description VARCHAR(255),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO message_templates (key, body, description) VALUES
('bot_welcome', E'Привет! Здесь покупают и продают авто в Приднестровье.\nВсе объявления рядом — листай, выбирай, пиши продавцу.', 'Приветствие бота при /start'),
('share_listing', '{title} — {price}', 'Текст при шеринге объявления'),
('share_profile', '{name} — AutoMarket', 'Текст при шеринге профиля'),
('notif_price_drop', E'<b>Цена снижена!</b>\n\n<b>{title}</b>\n<s>{old_price}</s> → <b>{new_price}</b>', 'Уведомление о снижении цены'),
('notif_new_by_search', E'<b>Новое по запросу «{search_name}»</b>\n\n{listings}', 'Новые по сохранённому поиску'),
('notif_seller_stats', E'<b>Статистика за неделю</b>\n\n🚗 <b>{title}</b>\n👁 {views} просмотров\n❤️ {likes} лайков', 'Статистика для продавца'),
('notif_listing_expiring', E'⏰ <b>Объявление истекает через {days} дн.</b>\n\n<b>{title}</b> — {price}', 'Истечение объявления'),
('saved_search_created', E'✅ <b>Поиск сохранён!</b>\n\n«{name}»\n\nВы будете получать уведомления о новых объявлениях, подходящих под ваши критерии.', 'Подтверждение сохранения поиска');
