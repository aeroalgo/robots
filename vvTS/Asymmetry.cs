using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000007 RID: 7
	[HandlerCategory("vvIndicators"), HandlerDecimals(2), HandlerName("Asymmetry")]
	public class Asymmetry : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x0600002C RID: 44 RVA: 0x00002BE9 File Offset: 0x00000DE9
		public IList<double> Execute(IList<double> src)
		{
			return Asymmetry.GenAsymmetry(src, this.Context, this.AsPeriod, 0, 0);
		}

		// Token: 0x0600002B RID: 43 RVA: 0x00002AA8 File Offset: 0x00000CA8
		public static IList<double> GenAsymmetry(IList<double> src, IContext ctx, int _AsPeriod, int _Smooth = 0, int _SmoothPhase = 0)
		{
			int count = src.Count;
			double[] array = new double[count];
			IList<double> result = array;
			double num = 0.0;
			double num2 = 0.0;
			double num3 = 0.0;
			for (int i = 0; i < count; i++)
			{
				if (i < _AsPeriod)
				{
					array[i] = 0.0;
				}
				else
				{
					for (int j = 0; j < _AsPeriod; j++)
					{
						num += src[i - j];
					}
					num /= (double)(_AsPeriod + 2);
					for (int k = 0; k < _AsPeriod; k++)
					{
						num2 += (src[i - k] - num) * (src[i - k] - num);
					}
					num2 /= (double)(_AsPeriod + 2);
					double num4 = Math.Sqrt(num2);
					for (int l = 0; l < _AsPeriod; l++)
					{
						num3 += (src[i - l] - num2) * (src[i - l] - num2);
					}
					num3 /= (double)(_AsPeriod + 2);
					array[i] = Math.Abs(num3 / (num2 * num4 * 100.0));
				}
			}
			if (_Smooth > 0)
			{
				result = JMA.GenJMA(array, _Smooth, _SmoothPhase);
			}
			return result;
		}

		// Token: 0x1700000D RID: 13
		[HandlerParameter(true, "15", Min = "5", Max = "60", Step = "1")]
		public int AsPeriod
		{
			// Token: 0x06000029 RID: 41 RVA: 0x00002A96 File Offset: 0x00000C96
			get;
			// Token: 0x0600002A RID: 42 RVA: 0x00002A9E File Offset: 0x00000C9E
			set;
		}

		// Token: 0x1700000E RID: 14
		public IContext Context
		{
			// Token: 0x0600002D RID: 45 RVA: 0x00002BFF File Offset: 0x00000DFF
			get;
			// Token: 0x0600002E RID: 46 RVA: 0x00002C07 File Offset: 0x00000E07
			set;
		}
	}
}
