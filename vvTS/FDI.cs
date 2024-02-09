using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000027 RID: 39
	[HandlerCategory("vvIndicators"), HandlerDecimals(2), HandlerName("FDI (Fractal Dimension Index)")]
	public class FDI : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x0600016E RID: 366 RVA: 0x000069E0 File Offset: 0x00004BE0
		private static double CalcFDI(IList<double> src, int period, bool lweighted, int shift)
		{
			double num = vvSeries.iHighest(src, shift, period);
			double num2 = vvSeries.iLowest(src, shift, period);
			double num3 = num - num2;
			double num4 = 0.0;
			for (int i = period - 2; i >= 0; i--)
			{
				double num5;
				if (num3 == 0.0)
				{
					num5 = 1.0;
				}
				else
				{
					num5 = (src[shift - i] - src[shift - i - 1]) / num3;
				}
				if (lweighted)
				{
					num4 += (double)(period - 1 - i) * Math.Sqrt(num5 * num5 + 1.0 / (double)(period * period));
				}
				else
				{
					num4 += Math.Sqrt(num5 * num5 + 1.0 / (double)(period * period));
				}
			}
			if (lweighted)
			{
				num4 /= (double)period / 2.0;
			}
			return 1.0 + Math.Log(2.0 * num4) / Math.Log(2.0 * (double)period);
		}

		// Token: 0x0600016F RID: 367 RVA: 0x00006B28 File Offset: 0x00004D28
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("FDI", new string[]
			{
				this.Length.ToString(),
				this.Smooth.ToString(),
				this.LinearWeigthed.ToString(),
				src.GetHashCode().ToString()
			}, () => FDI.GenFDI_LK(src, this.Context, this.Length, this.Smooth, this.LinearWeigthed));
		}

		// Token: 0x0600016D RID: 365 RVA: 0x00006974 File Offset: 0x00004B74
		public static IList<double> GenFDI_LK(IList<double> src, IContext context, int _Length, int _Smooth, bool _LWeigthed)
		{
			int count = src.Count;
			if (_Smooth < 1)
			{
				_Smooth = 1;
			}
			double[] array = new double[count];
			double num = 2.0 / ((double)_Smooth + 1.0);
			for (int i = _Length; i < count; i++)
			{
				double num2 = FDI.CalcFDI(src, _Length, _LWeigthed, i);
				array[i] = (1.0 - num) * array[i - 1] + num * num2;
			}
			return array;
		}

		// Token: 0x1700007B RID: 123
		public IContext Context
		{
			// Token: 0x06000170 RID: 368 RVA: 0x00006BB8 File Offset: 0x00004DB8
			get;
			// Token: 0x06000171 RID: 369 RVA: 0x00006BC0 File Offset: 0x00004DC0
			set;
		}

		// Token: 0x17000078 RID: 120
		[HandlerParameter(true, "60", Min = "2", Max = "100", Step = "2")]
		public int Length
		{
			// Token: 0x06000167 RID: 359 RVA: 0x0000693F File Offset: 0x00004B3F
			get;
			// Token: 0x06000168 RID: 360 RVA: 0x00006947 File Offset: 0x00004B47
			set;
		}

		// Token: 0x1700007A RID: 122
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool LinearWeigthed
		{
			// Token: 0x0600016B RID: 363 RVA: 0x00006961 File Offset: 0x00004B61
			get;
			// Token: 0x0600016C RID: 364 RVA: 0x00006969 File Offset: 0x00004B69
			set;
		}

		// Token: 0x17000079 RID: 121
		[HandlerParameter(true, "5", Min = "1", Max = "10", Step = "1")]
		public int Smooth
		{
			// Token: 0x06000169 RID: 361 RVA: 0x00006950 File Offset: 0x00004B50
			get;
			// Token: 0x0600016A RID: 362 RVA: 0x00006958 File Offset: 0x00004B58
			set;
		}
	}
}
