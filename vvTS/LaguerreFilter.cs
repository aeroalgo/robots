using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000032 RID: 50
	[HandlerCategory("vvIndicators"), HandlerDecimals(2), HandlerName("LaguerreFilter")]
	public class LaguerreFilter : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060001C4 RID: 452 RVA: 0x0000893D File Offset: 0x00006B3D
		public IList<double> Execute(IList<double> src)
		{
			return this.GenLaguerreFilter(src, this.Gamma);
		}

		// Token: 0x060001C3 RID: 451 RVA: 0x0000877C File Offset: 0x0000697C
		public IList<double> GenLaguerreFilter(IList<double> _src, double _gamma)
		{
			double[] array = new double[_src.Count];
			double[] array2 = new double[_src.Count];
			double[] array3 = new double[_src.Count];
			double[] array4 = new double[_src.Count];
			double[] array5 = new double[_src.Count];
			for (int i = 1; i < _src.Count; i++)
			{
				array2[i] = (1.0 - _gamma) * _src[i] + _gamma * array2[i - 1];
				array3[i] = -_gamma * array2[i] + array2[i - 1] + _gamma * array3[i - 1];
				array4[i] = -_gamma * array3[i] + array3[i - 1] + _gamma * array4[i - 1];
				array5[i] = -_gamma * array4[i] + array4[i - 1] + _gamma * array5[i - 1];
				double num = 0.0;
				double num2 = 0.0;
				if (array2[i] >= array3[i])
				{
					num = array2[i] - array3[i];
				}
				else
				{
					num2 = array3[i] - array2[i];
				}
				if (array3[i] >= array4[i])
				{
					num = num + array3[i] - array4[i];
				}
				else
				{
					num2 = num2 + array4[i] - array3[i];
				}
				if (array4[i] >= array5[i])
				{
					num = num + array4[i] - array5[i];
				}
				else
				{
					num2 = num2 + array5[i] - array4[i];
				}
				if (num + num2 != 0.0)
				{
					array[i] = (array2[i] + 2.0 * array3[i] + 2.0 * array4[i] + array5[i]) / 6.0;
				}
			}
			return array;
		}

		// Token: 0x17000097 RID: 151
		public IContext Context
		{
			// Token: 0x060001C5 RID: 453 RVA: 0x0000894C File Offset: 0x00006B4C
			get;
			// Token: 0x060001C6 RID: 454 RVA: 0x00008954 File Offset: 0x00006B54
			set;
		}

		// Token: 0x17000096 RID: 150
		[HandlerParameter(true, "0.7", Min = "0", Max = "1", Step = "0.1")]
		public double Gamma
		{
			// Token: 0x060001C1 RID: 449 RVA: 0x0000876A File Offset: 0x0000696A
			get;
			// Token: 0x060001C2 RID: 450 RVA: 0x00008772 File Offset: 0x00006972
			set;
		}
	}
}
