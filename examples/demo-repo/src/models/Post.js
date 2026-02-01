/**
 * Post Model
 * 
 * Defines the Post model with validations, hooks, and associations.
 */

const { DataTypes, Model } = require('sequelize');
const { Op } = require('sequelize');
const sequelize = require('../config/database');

class Post extends Model {
  /**
   * Check if a post with the given title exists
   * @param {string} title - The post title
   * @param {ObjectId} [excludePostId] - The id of the post to be excluded
   * @returns {Promise<boolean>}
   */
  static async isTitleTaken(title, excludePostId) {
    const post = await this.findOne({
      where: {
        title,
        ...(excludePostId && { id: { [Op.ne]: excludePostId } })
      }
    });
    return !!post;
  }

  /**
   * Get posts by author
   * @param {ObjectId} authorId - The author's ID
   * @returns {Promise<Array>}
   */
  static async getByAuthor(authorId) {
    return await this.findAll({
      where: { authorId },
      include: [{
        model: this.sequelize.models.User,
        as: 'author',
        attributes: ['id', 'username', 'email']
      }],
      order: [['createdAt', 'DESC']]
    });
  }

  /**
   * Get published posts
   * @returns {Promise<Array>}
   */
  static async getPublished() {
    return await this.findAll({
      where: { published: true },
      include: [{
        model: this.sequelize.models.User,
        as: 'author',
        attributes: ['id', 'username', 'email']
      }],
      order: [['publishedAt', 'DESC']],
      limit: 20
    });
  }

  /**
   * Search posts by keyword
   * @param {string} keyword - The search keyword
   * @returns {Promise<Array>}
   */
  static async searchByKeyword(keyword) {
    return await this.findAll({
      where: {
        [Op.or]: [
          { title: { [Op.iLike]: `%${keyword}%` } },
          { content: { [Op.iLike]: `%${keyword}%` } }
        ],
        published: true
      },
      include: [{
        model: this.sequelize.models.User,
        as: 'author',
        attributes: ['id', 'username', 'email']
      }],
      order: [['publishedAt', 'DESC']]
    });
  }
}

Post.init({
  id: {
    type: DataTypes.UUID,
    defaultValue: DataTypes.UUIDV4,
    primaryKey: true
  },
  title: {
    type: DataTypes.STRING,
    allowNull: false,
    validate: {
      len: [1, 200],
      notEmpty: true
    }
  },
  content: {
    type: DataTypes.TEXT,
    allowNull: false,
    validate: {
      len: [1, 10000],
      notEmpty: true
    }
  },
  excerpt: {
    type: DataTypes.TEXT,
    allowNull: true,
    validate: {
      len: [0, 500]
    }
  },
  authorId: {
    type: DataTypes.UUID,
    allowNull: false,
    references: {
      model: 'users',
      key: 'id'
    },
    onDelete: 'CASCADE',
    onUpdate: 'CASCADE'
  },
  published: {
    type: DataTypes.BOOLEAN,
    defaultValue: false,
    allowNull: false
  },
  publishedAt: {
    type: DataTypes.DATE,
    allowNull: true
  },
  featured: {
    type: DataTypes.BOOLEAN,
    defaultValue: false,
    allowNull: false
  },
  category: {
    type: DataTypes.STRING,
    allowNull: true,
    validate: {
      len: [0, 50]
    }
  },
  tags: {
    type: DataTypes.ARRAY(DataTypes.STRING),
    allowNull: true,
    defaultValue: []
  },
  slug: {
    type: DataTypes.STRING,
    allowNull: true,
    unique: true,
    validate: {
      len: [0, 200]
    }
  },
  viewCount: {
    type: DataTypes.INTEGER,
    defaultValue: 0,
    allowNull: false,
    validate: {
      min: 0
    }
  },
  likeCount: {
    type: DataTypes.INTEGER,
    defaultValue: 0,
    allowNull: false,
    validate: {
      min: 0
    }
  },
  commentCount: {
    type: DataTypes.INTEGER,
    defaultValue: 0,
    allowNull: false,
    validate: {
      min: 0
    }
  },
  seoTitle: {
    type: DataTypes.STRING,
    allowNull: true,
    validate: {
      len: [0, 70]
    }
  },
  seoDescription: {
    type: DataTypes.STRING,
    allowNull: true,
    validate: {
      len: [0, 160]
    }
  },
  seoKeywords: {
    type: DataTypes.ARRAY(DataTypes.STRING),
    allowNull: true,
    defaultValue: []
  },
  image: {
    type: DataTypes.STRING,
    allowNull: true
  },
  video: {
    type: DataTypes.STRING,
    allowNull: true
  },
  audio: {
    type: DataTypes.STRING,
    allowNull: true
  },
  language: {
    type: DataTypes.STRING,
    defaultValue: 'en',
    allowNull: false
  },
  readingTime: {
    type: DataTypes.INTEGER,
    defaultValue: 0,
    allowNull: false,
    validate: {
      min: 0
    }
  },
  wordCount: {
    type: DataTypes.INTEGER,
    defaultValue: 0,
    allowNull: false,
    validate: {
      min: 0
    }
  },
  status: {
    type: DataTypes.ENUM('draft', 'published', 'archived'),
    defaultValue: 'draft',
    allowNull: false
  },
  scheduledAt: {
    type: DataTypes.DATE,
    allowNull: true
  },
  password: {
    type: DataTypes.STRING,
    allowNull: true,
    validate: {
      len: [0, 128]
    }
  },
  allowComments: {
    type: DataTypes.BOOLEAN,
    defaultValue: true,
    allowNull: false
  },
  allowSharing: {
    type: DataTypes.BOOLEAN,
    defaultValue: true,
    allowNull: false
  },
  createdAt: {
    type: DataTypes.DATE,
    defaultValue: DataTypes.NOW,
    allowNull: false
  },
  updatedAt: {
    type: DataTypes.DATE,
    defaultValue: DataTypes.NOW,
    allowNull: false
  },
  publishedAt: {
    type: DataTypes.DATE,
    allowNull: true
  }
}, {
  sequelize,
  modelName: 'Post',
  tableName: 'posts',
  timestamps: true,
  paranoid: true, // Enable soft deletes
  indexes: [
    {
      fields: ['authorId']
    },
    {
      fields: ['published']
    },
    {
      fields: ['featured']
    },
    {
      fields: ['category']
    },
    {
      fields: ['publishedAt']
    },
    {
      fields: ['createdAt']
    },
    {
      fields: ['slug'],
      unique: true
    },
    {
      fields: ['status']
    },
    {
      fields: ['language']
    },
    {
      fields: ['viewCount']
    }
  ],
  hooks: {
    beforeCreate: async (post) => {
      // Generate slug from title if not provided
      if (!post.slug) {
        post.slug = post.title
          .toLowerCase()
          .replace(/[^\w\s-]/g, '')
          .replace(/[\s_-]+/g, '-')
          .replace(/^-+|-+$/g, '');
      }
      
      // Generate excerpt if not provided
      if (!post.excerpt && post.content) {
        post.excerpt = post.content.substring(0, 150) + '...';
      }
      
      // Calculate reading time (average 200 words per minute)
      if (post.content) {
        const wordCount = post.content.trim().split(/\s+/).length;
        post.readingTime = Math.ceil(wordCount / 200);
        post.wordCount = wordCount;
      }
      
      // Set publishedAt if post is published
      if (post.published && !post.publishedAt) {
        post.publishedAt = new Date();
      }
    },
    
    beforeUpdate: async (post) => {
      // Update slug if title changes
      if (post.changed('title') && !post.changed('slug')) {
        post.slug = post.title
          .toLowerCase()
          .replace(/[^\w\s-]/g, '')
          .replace(/[\s_-]+/g, '-')
          .replace(/^-+|-+$/g, '');
      }
      
      // Update excerpt if content changes
      if (post.changed('content') && !post.changed('excerpt')) {
        post.excerpt = post.content.substring(0, 150) + '...';
      }
      
      // Recalculate reading time if content changes
      if (post.changed('content')) {
        const wordCount = post.content.trim().split(/\s+/).length;
        post.readingTime = Math.ceil(wordCount / 200);
        post.wordCount = wordCount;
      }
      
      // Set publishedAt if post is published and wasn't before
      if (post.changed('published') && post.published && !post.publishedAt) {
        post.publishedAt = new Date();
      }
      
      // Update status based on published field
      if (post.changed('published')) {
        post.status = post.published ? 'published' : 'draft';
      }
    }
  }
});

// Define associations
Post.associate = (models) => {
  // A post belongs to a user (author)
  Post.belongsTo(models.User, {
    foreignKey: 'authorId',
    as: 'author',
    onDelete: 'CASCADE'
  });
  
  // A post can have many comments
  Post.hasMany(models.Comment, {
    foreignKey: 'postId',
    as: 'comments'
  });
  
  // A post can have many likes
  Post.hasMany(models.Like, {
    foreignKey: 'postId',
    as: 'likes'
  });
  
  // A post can have many views
  Post.hasMany(models.View, {
    foreignKey: 'postId',
    as: 'views'
  });
  
  // A post can have many shares
  Post.hasMany(models.Share, {
    foreignKey: 'postId',
    as: 'shares'
  });
  
  // A post can have many tags (many-to-many)
  Post.belongsToMany(models.Tag, {
    through: 'PostTags',
    as: 'tags',
    foreignKey: 'postId',
    otherKey: 'tagId'
  });
};

module.exports = Post;