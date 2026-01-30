/**
 * Secure Web Application Server
 * 
 * This is the main server file for the secure web application.
 * It implements security best practices and demonstrates secure coding patterns.
 */

const express = require('express');
const cors = require('cors');
const helmet = require('helmet');
const rateLimit = require('express-rate-limit');
const session = require('express-session');
const MongoStore = require('connect-mongo');
const path = require('path');
const fs = require('fs');
const https = require('https');
const http = require('http');
const compression = require('compression');
const cookieParser = require('cookie-parser');
const mongoSanitize = require('express-mongo-sanitize');
const xss = require('xss-clean');
const hpp = require('hpp');
const csurf = require('csurf');
const flash = require('express-flash');
const methodOverride = require('method-override');
const passport = require('passport');
const passportJWT = require('passport-jwt');
const passportLocal = require('passport-local');
const bcrypt = require('bcrypt');
const jwt = require('jsonwebtoken');
const validator = require('validator');
const crypto = require('crypto');
const axios = require('axios');
const validatorjs = require('validator');
const moment = require('moment');
const momentTimezone = require('moment-timezone');
const expressValidator = require('express-validator');
const uuid = require('uuid');
const winston = require('winston');
const redis = require('redis');
const socketIo = require('socket.io');
const swaggerUi = require('swagger-ui-express');
const swaggerJsdoc = require('swagger-jsdoc');
const joi = require('joi');
const joiPasswordComplexity = require('joi-password-complexity');
const multer = require('multer');
const upload = multer({ dest: 'uploads/' });
const sharp = require('sharp');
const NodeCache = require('node-cache');
const pg = require('pg');
const dotenv = require('dotenv');
const Sequelize = require('sequelize');
const sequelize = new Sequelize(process.env.DATABASE_URL);
const { Op } = require('sequelize');

// Load environment variables
dotenv.config();

// Initialize Express app
const app = express();

// Security middleware
app.use(helmet({
  contentSecurityPolicy: {
    directives: {
      defaultSrc: ["'self'"],
      styleSrc: ["'self'", "'unsafe-inline'"],
      scriptSrc: ["'self'"],
      imgSrc: ["'self'", "data:", "https:"],
      connectSrc: ["'self'", "https://api.example.com"],
      fontSrc: ["'self'", "https:", "data:"],
      objectSrc: ["'none'"],
      mediaSrc: ["'self'"],
      frameSrc: ["'none'"],
    },
  },
  hsts: {
    maxAge: 31536000,
    includeSubDomains: true,
    preload: true,
  },
  referrerPolicy: {
    policy: 'no-referrer',
  },
}));

// Rate limiting
const limiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100, // Limit each IP to 100 requests per windowMs
  message: 'Too many requests from this IP, please try again later.',
  standardHeaders: true, // Return rate limit info in the `RateLimit-*` headers
  legacyHeaders: false, // Disable the `X-RateLimit-*` headers
});
app.use(limiter);

// Body parsing middleware
app.use(express.json({ limit: '10mb' }));
app.use(express.urlencoded({ extended: true, limit: '10mb' }));

// Cookie parser
app.use(cookieParser(process.env.COOKIE_SECRET || 'fallback-secret'));

// Session management
app.use(session({
  secret: process.env.SESSION_SECRET || 'fallback-secret',
  resave: false,
  saveUninitialized: false,
  store: MongoStore.create({
    mongoUrl: process.env.MONGODB_URI || 'mongodb://localhost:27017/secure-app',
    touchAfter: 24 * 3600 // Time period in seconds
  }),
  cookie: {
    secure: process.env.NODE_ENV === 'production', // Set to true in production (HTTPS)
    httpOnly: true, // Prevent XSS attacks
    maxAge: 24 * 60 * 60 * 1000 // 24 hours
  }
}));

// Passport initialization
app.use(passport.initialize());
app.use(passport.session());

// Compression
app.use(compression());

// Static files
app.use(express.static(path.join(__dirname, 'public')));

// MongoDB sanitization
app.use(mongoSanitize());

// XSS protection
app.use(xss());

// HTTP parameter pollution protection
app.use(hpp());

// Method override
app.use(methodOverride('_method'));

// Flash messages
app.use(flash());

// CORS configuration
app.use(cors({
  origin: process.env.ALLOWED_ORIGINS ? process.env.ALLOWED_ORIGINS.split(',') : 'http://localhost:3000',
  credentials: true,
  optionsSuccessStatus: 200
}));

// Logging middleware
const logger = winston.createLogger({
  level: 'info',
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.errors({ stack: true }),
    winston.format.splat(),
    winston.format.json()
  ),
  defaultMeta: { service: 'secure-web-app' },
  transports: [
    new winston.transports.File({ filename: 'logs/error.log', level: 'error' }),
    new winston.transports.File({ filename: 'logs/combined.log' }),
  ],
});

if (process.env.NODE_ENV !== 'production') {
  logger.add(new winston.transports.Console({
    format: winston.format.simple(),
  }));
}

// Request logging
app.use((req, res, next) => {
  logger.info(`${req.method} ${req.url}`, {
    ip: req.ip,
    userAgent: req.get('User-Agent'),
    userId: req.user ? req.user.id : null
  });
  next();
});

// Database connection
const db = new pg.Client({
  connectionString: process.env.DATABASE_URL,
  ssl: process.env.NODE_ENV === 'production' ? { rejectUnauthorized: false } : false
});

db.connect()
  .then(() => console.log('Connected to PostgreSQL database'))
  .catch(err => console.error('Database connection error:', err));

// User model definition
const User = sequelize.define('User', {
  id: {
    type: Sequelize.UUID,
    defaultValue: Sequelize.UUIDV4,
    primaryKey: true
  },
  username: {
    type: Sequelize.STRING,
    allowNull: false,
    unique: true,
    validate: {
      len: [3, 30],
      isAlphanumeric: true
    }
  },
  email: {
    type: Sequelize.STRING,
    allowNull: false,
    unique: true,
    validate: {
      isEmail: true
    }
  },
  password: {
    type: Sequelize.STRING,
    allowNull: false,
    validate: {
      len: [8, 100]
    }
  },
  role: {
    type: Sequelize.ENUM('user', 'admin', 'moderator'),
    defaultValue: 'user'
  },
  isActive: {
    type: Sequelize.BOOLEAN,
    defaultValue: true
  },
  lastLogin: {
    type: Sequelize.DATE
  }
});

// Post model definition
const Post = sequelize.define('Post', {
  id: {
    type: Sequelize.UUID,
    defaultValue: Sequelize.UUIDV4,
    primaryKey: true
  },
  title: {
    type: Sequelize.STRING,
    allowNull: false,
    validate: {
      len: [1, 200]
    }
  },
  content: {
    type: Sequelize.TEXT,
    allowNull: false,
    validate: {
      len: [1, 10000]
    }
  },
  authorId: {
    type: Sequelize.UUID,
    allowNull: false,
    references: {
      model: User,
      key: 'id'
    }
  },
  published: {
    type: Sequelize.BOOLEAN,
    defaultValue: false
  },
  publishedAt: {
    type: Sequelize.DATE
  }
});

// Define associations
User.hasMany(Post, { foreignKey: 'authorId' });
Post.belongsTo(User, { foreignKey: 'authorId' });

// Validation schemas
const registerSchema = joi.object({
  username: joi.string().alphanum().min(3).max(30).required(),
  email: joi.string().email().required(),
  password: joiPasswordComplexity().required(),
  confirmPassword: joi.ref('password')
});

const loginSchema = joi.object({
  email: joi.string().email().required(),
  password: joi.string().required()
});

const postSchema = joi.object({
  title: joi.string().max(200).required(),
  content: joi.string().max(10000).required(),
  published: joi.boolean()
});

// Passport local strategy
passport.use(new passportLocal.Strategy(
  async (username, password, done) => {
    try {
      // Find user by username or email
      const user = await User.findOne({
        where: {
          [Op.or]: [
            { username: username },
            { email: username }
          ]
        }
      });

      if (!user) {
        return done(null, false, { message: 'Incorrect username or email.' });
      }

      // Verify password
      const isValidPassword = await bcrypt.compare(password, user.password);
      if (!isValidPassword) {
        return done(null, false, { message: 'Incorrect password.' });
      }

      // Update last login
      await user.update({ lastLogin: new Date() });

      return done(null, user);
    } catch (err) {
      return done(err);
    }
  }
));

// Passport JWT strategy
const jwtOptions = {
  jwtFromRequest: passportJWT.ExtractJwt.fromAuthHeaderAsBearerToken(),
  secretOrKey: process.env.JWT_SECRET || 'fallback-jwt-secret'
};

passport.use(new passportJWT.Strategy(jwtOptions, async (payload, done) => {
  try {
    const user = await User.findByPk(payload.sub);
    if (user) {
      return done(null, user);
    } else {
      return done(null, false);
    }
  } catch (err) {
    return done(err, false);
  }
}));

// Serialize user for session
passport.serializeUser((user, done) => {
  done(null, user.id);
});

// Deserialize user from session
passport.deserializeUser(async (id, done) => {
  try {
    const user = await User.findByPk(id);
    done(null, user);
  } catch (err) {
    done(err, null);
  }
});

// Middleware to check if user is authenticated
const isAuthenticated = (req, res, next) => {
  if (req.isAuthenticated()) {
    return next();
  }
  res.status(401).json({ error: 'Authentication required' });
};

// Middleware to check if user is admin
const isAdmin = (req, res, next) => {
  if (req.user && req.user.role === 'admin') {
    return next();
  }
  res.status(403).json({ error: 'Admin access required' });
};

// API Routes

// Authentication routes
app.post('/api/auth/register', async (req, res) => {
  try {
    // Validate input
    const { error, value } = registerSchema.validate(req.body);
    if (error) {
      return res.status(400).json({ error: error.details[0].message });
    }

    const { username, email, password } = value;

    // Check if user already exists
    const existingUser = await User.findOne({
      where: {
        [Op.or]: [
          { username: username },
          { email: email }
        ]
      }
    });

    if (existingUser) {
      return res.status(409).json({ error: 'Username or email already exists' });
    }

    // Hash password
    const saltRounds = parseInt(process.env.BCRYPT_ROUNDS) || 12;
    const hashedPassword = await bcrypt.hash(password, saltRounds);

    // Create user
    const user = await User.create({
      username,
      email,
      password: hashedPassword
    });

    // Create JWT token
    const token = jwt.sign(
      { sub: user.id, username: user.username },
      process.env.JWT_SECRET || 'fallback-jwt-secret',
      { expiresIn: '1h' }
    );

    res.status(201).json({
      message: 'User registered successfully',
      token,
      user: {
        id: user.id,
        username: user.username,
        email: user.email
      }
    });
  } catch (err) {
    logger.error('Registration error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

app.post('/api/auth/login', async (req, res) => {
  try {
    // Validate input
    const { error, value } = loginSchema.validate(req.body);
    if (error) {
      return res.status(400).json({ error: error.details[0].message });
    }

    const { email, password } = value;

    // Find user
    const user = await User.findOne({ where: { email } });
    if (!user) {
      return res.status(401).json({ error: 'Invalid credentials' });
    }

    // Verify password
    const isValidPassword = await bcrypt.compare(password, user.password);
    if (!isValidPassword) {
      return res.status(401).json({ error: 'Invalid credentials' });
    }

    // Update last login
    await user.update({ lastLogin: new Date() });

    // Create JWT token
    const token = jwt.sign(
      { sub: user.id, username: user.username },
      process.env.JWT_SECRET || 'fallback-jwt-secret',
      { expiresIn: '1h' }
    );

    res.json({
      message: 'Login successful',
      token,
      user: {
        id: user.id,
        username: user.username,
        email: user.email,
        role: user.role
      }
    });
  } catch (err) {
    logger.error('Login error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

app.post('/api/auth/logout', isAuthenticated, (req, res) => {
  req.logout((err) => {
    if (err) {
      logger.error('Logout error:', err);
      return res.status(500).json({ error: 'Logout failed' });
    }
    req.session.destroy((err) => {
      if (err) {
        logger.error('Session destroy error:', err);
        return res.status(500).json({ error: 'Logout failed' });
      }
      res.json({ message: 'Logged out successfully' });
    });
  });
});

// User management routes
app.get('/api/users', isAdmin, async (req, res) => {
  try {
    const users = await User.findAll({
      attributes: { exclude: ['password'] },
      order: [['createdAt', 'DESC']]
    });
    res.json(users);
  } catch (err) {
    logger.error('Get users error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

app.get('/api/users/:id', isAuthenticated, async (req, res) => {
  try {
    const userId = req.params.id;
    
    // Check if user is requesting their own data or is admin
    if (req.user.id !== userId && req.user.role !== 'admin') {
      return res.status(403).json({ error: 'Access denied' });
    }

    const user = await User.findByPk(userId, {
      attributes: { exclude: ['password'] }
    });

    if (!user) {
      return res.status(404).json({ error: 'User not found' });
    }

    res.json(user);
  } catch (err) {
    logger.error('Get user error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

app.put('/api/users/:id', isAuthenticated, async (req, res) => {
  try {
    const userId = req.params.id;
    
    // Check if user is updating their own data or is admin
    if (req.user.id !== userId && req.user.role !== 'admin') {
      return res.status(403).json({ error: 'Access denied' });
    }

    const { username, email } = req.body;

    // Validate input
    if (username && !validator.isAlphanumeric(username)) {
      return res.status(400).json({ error: 'Username must be alphanumeric' });
    }

    if (email && !validator.isEmail(email)) {
      return res.status(400).json({ error: 'Invalid email format' });
    }

    const user = await User.findByPk(userId);
    if (!user) {
      return res.status(404).json({ error: 'User not found' });
    }

    // Update user
    await user.update({ username, email });

    res.json({
      message: 'User updated successfully',
      user: {
        id: user.id,
        username: user.username,
        email: user.email
      }
    });
  } catch (err) {
    logger.error('Update user error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

app.delete('/api/users/:id', isAdmin, async (req, res) => {
  try {
    const userId = req.params.id;

    const user = await User.findByPk(userId);
    if (!user) {
      return res.status(404).json({ error: 'User not found' });
    }

    await user.destroy();

    res.json({ message: 'User deleted successfully' });
  } catch (err) {
    logger.error('Delete user error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// Post routes
app.get('/api/posts', async (req, res) => {
  try {
    const posts = await Post.findAll({
      where: { published: true },
      include: [{
        model: User,
        attributes: ['id', 'username']
      }],
      order: [['publishedAt', 'DESC']]
    });
    res.json(posts);
  } catch (err) {
    logger.error('Get posts error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

app.get('/api/posts/:id', async (req, res) => {
  try {
    const postId = req.params.id;

    const post = await Post.findOne({
      where: { 
        id: postId,
        published: true 
      },
      include: [{
        model: User,
        attributes: ['id', 'username']
      }]
    });

    if (!post) {
      return res.status(404).json({ error: 'Post not found' });
    }

    res.json(post);
  } catch (err) {
    logger.error('Get post error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

app.post('/api/posts', isAuthenticated, async (req, res) => {
  try {
    // Validate input
    const { error, value } = postSchema.validate(req.body);
    if (error) {
      return res.status(400).json({ error: error.details[0].message });
    }

    const { title, content, published } = value;

    // Create post
    const post = await Post.create({
      title,
      content,
      authorId: req.user.id,
      published: published || false,
      publishedAt: published ? new Date() : null
    });

    res.status(201).json({
      message: 'Post created successfully',
      post: {
        id: post.id,
        title: post.title,
        content: post.content,
        authorId: post.authorId,
        published: post.published,
        publishedAt: post.publishedAt
      }
    });
  } catch (err) {
    logger.error('Create post error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

app.put('/api/posts/:id', isAuthenticated, async (req, res) => {
  try {
    const postId = req.params.id;

    // Validate input
    const { error, value } = postSchema.validate(req.body);
    if (error) {
      return res.status(400).json({ error: error.details[0].message });
    }

    const { title, content, published } = value;

    const post = await Post.findByPk(postId);
    if (!post) {
      return res.status(404).json({ error: 'Post not found' });
    }

    // Check if user is owner or admin
    if (post.authorId !== req.user.id && req.user.role !== 'admin') {
      return res.status(403).json({ error: 'Access denied' });
    }

    // Update post
    await post.update({
      title,
      content,
      published,
      publishedAt: published && !post.publishedAt ? new Date() : post.publishedAt
    });

    res.json({
      message: 'Post updated successfully',
      post: {
        id: post.id,
        title: post.title,
        content: post.content,
        authorId: post.authorId,
        published: post.published,
        publishedAt: post.publishedAt
      }
    });
  } catch (err) {
    logger.error('Update post error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

app.delete('/api/posts/:id', isAuthenticated, async (req, res) => {
  try {
    const postId = req.params.id;

    const post = await Post.findByPk(postId);
    if (!post) {
      return res.status(404).json({ error: 'Post not found' });
    }

    // Check if user is owner or admin
    if (post.authorId !== req.user.id && req.user.role !== 'admin') {
      return res.status(403).json({ error: 'Access denied' });
    }

    await post.destroy();

    res.json({ message: 'Post deleted successfully' });
  } catch (err) {
    logger.error('Delete post error:', err);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// Error handling middleware
app.use((err, req, res, next) => {
  logger.error('Unhandled error:', err);
  res.status(500).json({ error: 'Internal server error' });
});

// 404 handler
app.use('*', (req, res) => {
  res.status(404).json({ error: 'Route not found' });
});

// Start server
const PORT = process.env.PORT || 3000;

// Handle graceful shutdown
process.on('SIGTERM', () => {
  logger.info('SIGTERM received, shutting down gracefully');
  server.close(() => {
    logger.info('Process terminated');
  });
});

process.on('SIGINT', () => {
  logger.info('SIGINT received, shutting down gracefully');
  server.close(() => {
    logger.info('Process terminated');
  });
});

const server = app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
  logger.info(`Server started on port ${PORT}`);
});

// Initialize database
sequelize.sync()
  .then(() => {
    console.log('Database synchronized');
  })
  .catch(err => {
    console.error('Database synchronization error:', err);
    logger.error('Database synchronization error:', err);
  });

module.exports = app;