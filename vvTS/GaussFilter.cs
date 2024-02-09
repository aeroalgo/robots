using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000175 RID: 373
	[HandlerCategory("vvAverages"), HandlerName("Gaussian Filter")]
	public class GaussFilter : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000BC3 RID: 3011 RVA: 0x00032A10 File Offset: 0x00030C10
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("GaussianFilter", new string[]
			{
				this.Length.ToString(),
				this.Order.ToString(),
				src.GetHashCode().ToString()
			}, () => GaussFilter.GenGaussFilter(src, this.Length, this.Order));
		}

		// Token: 0x06000BC4 RID: 3012 RVA: 0x00032A8E File Offset: 0x00030C8E
		private static int fact(int n)
		{
			if (n == 1 || n == 0)
			{
				return 1;
			}
			return n * GaussFilter.fact(n - 1);
		}

		// Token: 0x06000BC2 RID: 3010 RVA: 0x00032844 File Offset: 0x00030A44
		public static IList<double> GenGaussFilter(IList<double> src, int _length, int _order)
		{
			int num = Math.Max(_length, 2);
			int num2 = Math.Max(Math.Min(_order, 12), 1);
			double[,] array = new double[num2 + 1, 3];
			double[] array2 = new double[src.Count];
			double num3 = 3.1415926535897931;
			double num4 = (1.0 - Math.Cos(2.0 * num3 / (double)num)) / (Math.Pow(Math.Sqrt(2.0), 2.0 / (double)num2) - 1.0);
			double num5 = -num4 + Math.Sqrt(num4 * num4 + 2.0 * num4);
			for (int i = 0; i <= num2; i++)
			{
				array[i, 0] = (double)(GaussFilter.fact(_order) / (GaussFilter.fact(_order - i) * GaussFilter.fact(i)));
				array[i, 1] = Math.Pow(num5, (double)i);
				array[i, 2] = Math.Pow(1.0 - num5, (double)i);
			}
			IList<double> list = SMA.GenSMA(src, 1);
			for (int j = num2; j < src.Count; j++)
			{
				array2[j] = list[j] * array[num2, 1];
				double num6 = 1.0;
				int k = 1;
				while (k <= num2)
				{
					array2[j] += num6 * array[k, 0] * array[k, 2] * array2[j - k];
					k++;
					num6 *= -1.0;
				}
			}
			return array2;
		}

		// Token: 0x170003DF RID: 991
		public IContext Context
		{
			// Token: 0x06000BC5 RID: 3013 RVA: 0x00032AA3 File Offset: 0x00030CA3
			get;
			// Token: 0x06000BC6 RID: 3014 RVA: 0x00032AAB File Offset: 0x00030CAB
			set;
		}

		// Token: 0x170003DD RID: 989
		[HandlerParameter(true, "20", Min = "1", Max = "50", Step = "1")]
		public int Length
		{
			// Token: 0x06000BBE RID: 3006 RVA: 0x00032821 File Offset: 0x00030A21
			get;
			// Token: 0x06000BBF RID: 3007 RVA: 0x00032829 File Offset: 0x00030A29
			set;
		}

		// Token: 0x170003DE RID: 990
		[HandlerParameter(true, "2", Min = "1", Max = "50", Step = "1")]
		public int Order
		{
			// Token: 0x06000BC0 RID: 3008 RVA: 0x00032832 File Offset: 0x00030A32
			get;
			// Token: 0x06000BC1 RID: 3009 RVA: 0x0003283A File Offset: 0x00030A3A
			set;
		}
	}
}
