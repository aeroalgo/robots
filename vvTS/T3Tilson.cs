using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020001A2 RID: 418
	[HandlerCategory("vvAverages"), HandlerName("T3")]
	public class T3Tilson : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000D3E RID: 3390 RVA: 0x0003A580 File Offset: 0x00038780
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("t3ma", new string[]
			{
				this.T3Period.ToString(),
				this.T3Hot.ToString(),
				this.T3Original.ToString(),
				src.GetHashCode().ToString()
			}, () => T3Tilson.GenT3(src, this.T3Period, this.T3Hot, this.T3Original));
		}

		// Token: 0x06000D3B RID: 3387 RVA: 0x0003A0A8 File Offset: 0x000382A8
		public static IList<double> GenT3(IList<double> src, int _T3Period, double _T3Hot, bool _T3Original)
		{
			int count = src.Count;
			double[] array = new double[count];
			double[,] emas = new double[count, 6];
			double c = -_T3Hot * _T3Hot * _T3Hot;
			double c2 = 3.0 * _T3Hot * _T3Hot + 3.0 * _T3Hot * _T3Hot * _T3Hot;
			double c3 = -6.0 * _T3Hot * _T3Hot - 3.0 * _T3Hot - 3.0 * _T3Hot * _T3Hot * _T3Hot;
			double c4 = 1.0 + 3.0 * _T3Hot + _T3Hot * _T3Hot * _T3Hot + 3.0 * _T3Hot * _T3Hot;
			_T3Period = Math.Max(1, _T3Period);
			double alpha;
			if (_T3Original)
			{
				alpha = 2.0 / (1.0 + (double)_T3Period);
			}
			else
			{
				alpha = 2.0 / (2.0 + ((double)_T3Period - 1.0) / 2.0);
			}
			for (int i = 0; i < count; i++)
			{
				array[i] = T3Tilson.iT3(src[i], i, alpha, emas, c, c2, c3, c4);
			}
			return array;
		}

		// Token: 0x06000D3D RID: 3389 RVA: 0x0003A348 File Offset: 0x00038548
		public static IList<double> GenT3orig(IList<double> src, IContext ctx, int _T3Period, double _T3Hot)
		{
			int count = src.Count;
			double[] array = new double[count];
			double num = 0.0;
			double num2 = 0.0;
			double num3 = 0.0;
			double num4 = 0.0;
			double num5 = 0.0;
			double num6 = 0.0;
			double num7 = _T3Hot * _T3Hot;
			double num8 = num7 * _T3Hot;
			double num9 = -num8;
			double num10 = 3.0 * (num7 + num8);
			double num11 = -3.0 * (2.0 * num7 + _T3Hot + num8);
			double num12 = 1.0 + 3.0 * _T3Hot + num8 + 3.0 * num7;
			double num13 = (double)_T3Period;
			if (num13 < 1.0)
			{
				num13 = 1.0;
			}
			num13 = 1.0 + 0.5 * (num13 - 1.0);
			double num14 = 2.0 / (num13 + 1.0);
			double num15 = 1.0 - num14;
			for (int i = 0; i < count; i++)
			{
				num = num14 * src[i] + num15 * num;
				num2 = num14 * num + num15 * num2;
				num3 = num14 * num2 + num15 * num3;
				num4 = num14 * num3 + num15 * num4;
				num5 = num14 * num4 + num15 * num5;
				num6 = num14 * num5 + num15 * num6;
				array[i] = num9 * num6 + num10 * num5 + num11 * num4 + num12 * num3;
			}
			return array;
		}

		// Token: 0x06000D3C RID: 3388 RVA: 0x0003A1E0 File Offset: 0x000383E0
		public static double iT3(double price, int shift, double alpha, double[,] emas, double c1, double c2, double c3, double c4)
		{
			if (shift <= 1)
			{
				emas[shift, 0] = price;
				emas[shift, 1] = price;
				emas[shift, 2] = price;
				emas[shift, 3] = price;
				emas[shift, 4] = price;
				emas[shift, 5] = price;
			}
			else
			{
				emas[shift, 0] = emas[shift - 1, 0] + alpha * (price - emas[shift - 1, 0]);
				emas[shift, 1] = emas[shift - 1, 1] + alpha * (emas[shift, 0] - emas[shift - 1, 1]);
				emas[shift, 2] = emas[shift - 1, 2] + alpha * (emas[shift, 1] - emas[shift - 1, 2]);
				emas[shift, 3] = emas[shift - 1, 3] + alpha * (emas[shift, 2] - emas[shift - 1, 3]);
				emas[shift, 4] = emas[shift - 1, 4] + alpha * (emas[shift, 3] - emas[shift - 1, 4]);
				emas[shift, 5] = emas[shift - 1, 5] + alpha * (emas[shift, 4] - emas[shift - 1, 5]);
			}
			return c1 * emas[shift, 5] + c2 * emas[shift, 4] + c3 * emas[shift, 3] + c4 * emas[shift, 2];
		}

		// Token: 0x1700044E RID: 1102
		public IContext Context
		{
			// Token: 0x06000D3F RID: 3391 RVA: 0x0003A610 File Offset: 0x00038810
			get;
			// Token: 0x06000D40 RID: 3392 RVA: 0x0003A618 File Offset: 0x00038818
			set;
		}

		// Token: 0x1700044C RID: 1100
		[HandlerParameter(true, "0.7", Min = "0", Max = "1", Step = "0.05")]
		public double T3Hot
		{
			// Token: 0x06000D37 RID: 3383 RVA: 0x0003A086 File Offset: 0x00038286
			get;
			// Token: 0x06000D38 RID: 3384 RVA: 0x0003A08E File Offset: 0x0003828E
			set;
		}

		// Token: 0x1700044D RID: 1101
		[HandlerParameter(true, "true", NotOptimized = true)]
		public bool T3Original
		{
			// Token: 0x06000D39 RID: 3385 RVA: 0x0003A097 File Offset: 0x00038297
			get;
			// Token: 0x06000D3A RID: 3386 RVA: 0x0003A09F File Offset: 0x0003829F
			set;
		}

		// Token: 0x1700044B RID: 1099
		[HandlerParameter(true, "14", Min = "5", Max = "80", Step = "1")]
		public int T3Period
		{
			// Token: 0x06000D35 RID: 3381 RVA: 0x0003A075 File Offset: 0x00038275
			get;
			// Token: 0x06000D36 RID: 3382 RVA: 0x0003A07D File Offset: 0x0003827D
			set;
		}
	}
}
