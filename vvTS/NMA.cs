using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200018F RID: 399
	[HandlerCategory("vvAverages"), HandlerName("Natural MA")]
	public class NMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000C99 RID: 3225 RVA: 0x000368FC File Offset: 0x00034AFC
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("NaturalMA", new string[]
			{
				this.NMAperiod.ToString(),
				this.Smooth.ToString(),
				src.GetHashCode().ToString()
			}, () => NMA.GenNMA(src, this.NMAperiod, this.Smooth));
		}

		// Token: 0x06000C98 RID: 3224 RVA: 0x00036700 File Offset: 0x00034900
		public static IList<double> GenNMA(IList<double> src, int _nmaperiod, int _smooth)
		{
			double[] array = new double[src.Count];
			double[] array2 = new double[src.Count];
			for (int i = _nmaperiod + 1; i < src.Count; i++)
			{
				double num = 0.0;
				double num2 = 0.0;
				double num3 = 0.0;
				for (int j = 0; j < _nmaperiod; j++)
				{
					double num4 = Math.Abs(Math.Log(src[i - j]) * 100000.0 - Math.Log(src[i - j - 1]) * 100000.0);
					num2 += num4;
					num3 += num4 * (Math.Sqrt((double)(j + 1)) - Math.Sqrt((double)j));
				}
				if (num2 != 0.0)
				{
					num = Math.Round(num3) / Math.Round(num2);
				}
				array2[i] = array2[i - 1] + num * (src[i] - array2[i - 1]);
				double num5 = 0.0;
				if (_smooth <= 0)
				{
					num5 = array2[i];
				}
				if (_smooth == 1)
				{
					num5 = (array2[i] + array2[i - 1] + array2[i - 2]) / 3.0;
				}
				if (_smooth == 2)
				{
					num5 = (array2[i] + 2.0 * array2[i - 1] + 2.0 * array2[i - 2] + array2[i - 3]) / 6.0;
				}
				if (_smooth >= 3)
				{
					num5 = (array2[i] + 2.0 * array2[i - 1] + 3.0 * array2[i - 2] + 3.0 * array2[i - 3] + 2.0 * array2[i - 4] + array2[i - 5]) / 12.0;
				}
				array[i] = num5;
			}
			return array;
		}

		// Token: 0x1700041D RID: 1053
		public IContext Context
		{
			// Token: 0x06000C9A RID: 3226 RVA: 0x0003697A File Offset: 0x00034B7A
			get;
			// Token: 0x06000C9B RID: 3227 RVA: 0x00036982 File Offset: 0x00034B82
			set;
		}

		// Token: 0x1700041B RID: 1051
		[HandlerParameter(true, "20", Min = "1", Max = "60", Step = "1")]
		public int NMAperiod
		{
			// Token: 0x06000C94 RID: 3220 RVA: 0x000366DD File Offset: 0x000348DD
			get;
			// Token: 0x06000C95 RID: 3221 RVA: 0x000366E5 File Offset: 0x000348E5
			set;
		}

		// Token: 0x1700041C RID: 1052
		[HandlerParameter(true, "1", Min = "1", Max = "9", Step = "1")]
		public int Smooth
		{
			// Token: 0x06000C96 RID: 3222 RVA: 0x000366EE File Offset: 0x000348EE
			get;
			// Token: 0x06000C97 RID: 3223 RVA: 0x000366F6 File Offset: 0x000348F6
			set;
		}
	}
}
