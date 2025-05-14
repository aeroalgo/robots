using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000137 RID: 311
	[HandlerCategory("vvRSI"), HandlerDecimals(2), HandlerName("LaguerreACS")]
	public class LaguerreACS : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x0600094D RID: 2381 RVA: 0x0002713B File Offset: 0x0002533B
		public IList<double> Execute(IList<double> src)
		{
			return LaguerreACS.GenLaguerreACS(src, this.Gamma, this.Smooth);
		}

		// Token: 0x0600094C RID: 2380 RVA: 0x00026F54 File Offset: 0x00025154
		public static IList<double> GenLaguerreACS(IList<double> _source, double _gamma, int _smooth)
		{
			int count = _source.Count;
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			double[] array5 = new double[count];
			double[] array6 = new double[count];
			for (int i = _smooth; i < count; i++)
			{
				array3[i] = (1.0 - _gamma) * _source[i] + _gamma * array3[i - 1];
				array4[i] = -_gamma * array3[i] + array3[i - 1] + _gamma * array4[i - 1];
				array5[i] = -_gamma * array4[i] + array4[i - 1] + _gamma * array5[i - 1];
				array6[i] = -_gamma * array5[i] + array5[i - 1] + _gamma * array6[i - 1];
				double num = 0.0;
				double num2 = 0.0;
				double num3 = 0.0;
				if (array3[i] >= array4[i])
				{
					num = array3[i] - array4[i];
				}
				else
				{
					num2 = array4[i] - array3[i];
				}
				if (array4[i] >= array5[i])
				{
					num = num + array4[i] - array5[i];
				}
				else
				{
					num2 = num2 + array5[i] - array4[i];
				}
				if (array5[i] >= array6[i])
				{
					num = num + array5[i] - array6[i];
				}
				else
				{
					num2 = num2 + array6[i] - array5[i];
				}
				if (num + num2 != 0.0)
				{
					array2[i] = num / (num + num2);
				}
				if (_smooth < 2)
				{
					array[i] = array2[i];
				}
				else
				{
					for (int j = i - _smooth; j < i; j++)
					{
						num3 += array2[j];
					}
					array[i] = num3 / (double)_smooth;
				}
			}
			return array;
		}

		// Token: 0x17000302 RID: 770
		public IContext Context
		{
			// Token: 0x0600094E RID: 2382 RVA: 0x0002714F File Offset: 0x0002534F
			get;
			// Token: 0x0600094F RID: 2383 RVA: 0x00027157 File Offset: 0x00025357
			set;
		}

		// Token: 0x17000300 RID: 768
		[HandlerParameter(true, "0.6", Min = "0", Max = "1", Step = "0.1")]
		public double Gamma
		{
			// Token: 0x06000948 RID: 2376 RVA: 0x00026F31 File Offset: 0x00025131
			get;
			// Token: 0x06000949 RID: 2377 RVA: 0x00026F39 File Offset: 0x00025139
			set;
		}

		// Token: 0x17000301 RID: 769
		[HandlerParameter(true, "2", Min = "0", Max = "20", Step = "1")]
		public int Smooth
		{
			// Token: 0x0600094A RID: 2378 RVA: 0x00026F42 File Offset: 0x00025142
			get;
			// Token: 0x0600094B RID: 2379 RVA: 0x00026F4A File Offset: 0x0002514A
			set;
		}
	}
}
